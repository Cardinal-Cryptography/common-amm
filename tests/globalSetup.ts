import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import Factory_factory from '../types/constructors/factory_contract';
import Pair_factory from '../types/constructors/pair_contract';
import Token_factory from '../types/constructors/psp22_token';
import Wnative_factory from '../types/constructors/wnative_contract';
import Router_factory from '../types/constructors/router_contract';
import Factory from '../types/contracts/factory_contract';
import Pair from '../types/contracts/pair_contract';
import Token from '../types/contracts/psp22_token';
import Wnative from '../types/contracts/wnative_contract';
import Router from '../types/contracts/router_contract';
import { KeyringPair } from '@polkadot/keyring/types';
import BN from 'bn.js';

// Create a new instance of contract
const wsProvider = new WsProvider('ws://127.0.0.1:9944');
// Create a keyring instance
const keyring = new Keyring({ type: 'sr25519' });
export default async function setupApi(): Promise<void> {
  const api = await ApiPromise.create({ provider: wsProvider });
  const alice = keyring.addFromUri('//Alice');
  const bob = keyring.addFromUri('//Bob');
  await api.tx.balances.transfer(bob.address, 1_000_000_000_000).signAndSend(alice);
  globalThis.setup = await setupContracts(api, alice, bob);
}

interface TestFixture {
  token0: Token;
  token1: Token;
  wnative: Wnative;
  router: Router;
  factory: Factory;
}

async function setupContracts(api: ApiPromise, deployer: KeyringPair, wallet: KeyringPair): Promise<TestFixture> {
  let pairFactory = new Pair_factory(api, deployer);
  let pair = new Pair((await pairFactory.new()).address, deployer, api);
  let pairHash = pair.abi.info.source.wasmHash.toHex();
  let factoryFactory = new Factory_factory(api, deployer);
  let factory = new Factory(
    (await factoryFactory.new(wallet.address, pairHash)).address,
    deployer,
    api,
  );
  let [token0, token1] = await setupPsp22(api, deployer);
  let [wnative, router] = await setupRouter(api, deployer, factory);
  return {
    token0,
    token1,
    wnative,
    router,
    factory,
  }
}

async function setupPsp22(api: ApiPromise, deployer: KeyringPair): Promise<Token[]> {
  let tokenFactory = new Token_factory(api, deployer);
  let totalSupply = new BN(10000000);

  let tokenAaddress = (
    await tokenFactory.new(
      totalSupply,
      'TOKEN_A' as unknown as string[],
      'TKNA' as unknown as string[],
      18,
    )
  ).address;
  let tokenBaddress = (
    await tokenFactory.new(
      totalSupply,
      'TOKEN_B' as unknown as string[],
      'TKNB' as unknown as string[],
      18,
    )
  ).address;
  let [token0Address, token1Address] =
    tokenAaddress > tokenBaddress
      ? [tokenBaddress, tokenAaddress]
      : [tokenAaddress, tokenBaddress];
  let token0 = new Token(token0Address, deployer, api);
  let token1 = new Token(token1Address, deployer, api);
  return [token0, token1]
}

async function setupRouter(api: ApiPromise, deployer: KeyringPair, factory: Factory): Promise<[Wnative, Router]> {
  let wnativeFactory = new Wnative_factory(api, deployer);
  let wnative = new Wnative((await wnativeFactory.new()).address, deployer, api);
  let routerFactory = new Router_factory(api, deployer);
  let router = new Router(
    (await routerFactory.new(factory.address, wnative.address)).address,
    deployer,
    api,
  );
  return [wnative, router]
}