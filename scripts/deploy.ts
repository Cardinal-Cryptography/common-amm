import fs from 'fs';
import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import type { Hash } from '@polkadot/types/interfaces/runtime';
import { Abi } from '@polkadot/api-contract';
import Token_factory from '../types/constructors/psp22_token';
import Factory_factory from '../types/constructors/factory_contract';
import Wnative_factory from '../types/constructors/wnative_contract';
import Router_factory from '../types/constructors/router_contract';
import Token from '../types/contracts/psp22_token';
import Factory from '../types/contracts/factory_contract';
import Wnative from '../types/contracts/wnative_contract';
import Router from '../types/contracts/router_contract';
import { TOTAL_SUPPLY, STABLE_TOTAL_SUPPLY, ONE_STABLECOIN } from './constants';
import { approveSpender, addLiquidityNative } from './utils';
import parseUnits from './shared';
import 'dotenv/config';
import '@polkadot/api-augment';

// Create a new instance of contract
const wsProvider = new WsProvider(process.env.WS_NODE);
// Create a keyring instance
const keyring = new Keyring({ type: 'sr25519' });

async function main(): Promise<void> {
  const api = await ApiPromise.create({ provider: wsProvider });
  const deployer = keyring.addFromUri(process.env.AUTHORITY_SEED);
  const tokenFactory = new Token_factory(api, deployer);
  const tokenContractRaw = JSON.parse(
    fs.readFileSync(__dirname + `/../artifacts/psp22_token.contract`, 'utf8'),
  );
  const tokenAbi = new Abi(tokenContractRaw);
  let { gasRequired } = await api.call.contractsApi.instantiate(
    deployer.address,
    0,
    null,
    null,
    { Upload: tokenAbi.info.source.wasm },
    tokenAbi.constructors[0].toU8a([TOTAL_SUPPLY, 'Apollo Token', 'APLO', 18]),
    '',
  );
  const { address: aploAddress } = await tokenFactory.new(
    TOTAL_SUPPLY,
    'Apollo Token',
    'APLO',
    18,
    { gasLimit: gasRequired },
  );
  console.log('aplo token address:', aploAddress);
  const aplo = new Token(aploAddress, deployer, api);
  const { address: usdcAddress } = await tokenFactory.new(
    STABLE_TOTAL_SUPPLY,
    'USD Coin',
    'USDC',
    6,
    { gasLimit: gasRequired },
  );
  console.log('usdc token address:', usdcAddress);
  const usdc = new Token(usdcAddress, deployer, api);
  const { address: usdtAddress } = await tokenFactory.new(
    STABLE_TOTAL_SUPPLY,
    'Tether USD',
    'USDT',
    6,
    { gasLimit: gasRequired },
  );
  console.log('usdt token address:', usdtAddress);
  const usdt = new Token(usdtAddress, deployer, api);

  const pairContractRaw = JSON.parse(
    fs.readFileSync(__dirname + `/../artifacts/pair_contract.contract`, 'utf8'),
  );
  const pairAbi = new Abi(pairContractRaw);
  const deployedHash: Hash = await (new Promise(async (resolve, reject) => {
    const unsub = await api.tx.contracts
      .uploadCode(pairAbi.info.source.wasm, null, 0)
      .signAndSend(deployer, (result) => {
        if (result.isFinalized) {
          unsub();
          resolve(result.txHash)
        }
        if (result.isError) {
          unsub();
          reject(result)
        }
      });
  }))
  console.log('pair deployed with', deployedHash.toHuman());
  const pairHash = pairAbi.info.source.wasmHash.toHex();

  const factoryContractRaw = JSON.parse(
    fs.readFileSync(
      __dirname + `/../artifacts/factory_contract.contract`,
      'utf8',
    ),
  );
  const factoryAbi = new Abi(factoryContractRaw);
  ({ gasRequired } = await api.call.contractsApi.instantiate(
    deployer.address,
    0,
    null,
    null,
    { Upload: factoryAbi.info.source.wasm },
    factoryAbi.constructors[0].toU8a([deployer.address, pairHash]),
    '',
  ));
  const factoryFactory = new Factory_factory(api, deployer);
  const { address: factoryAddress } = await factoryFactory.new(
    deployer.address,
    pairHash,
    { gasLimit: gasRequired },
  );
  console.log('factory address:', factoryAddress);
  const factory = new Factory(factoryAddress, deployer, api);

  const wnativeContractRaw = JSON.parse(
    fs.readFileSync(
      __dirname + `/../artifacts/wnative_contract.contract`,
      'utf8',
    ),
  );
  const wnativeAbi = new Abi(wnativeContractRaw);
  ({ gasRequired } = await api.call.contractsApi.instantiate(
    deployer.address,
    0,
    null,
    null,
    { Upload: wnativeAbi.info.source.wasm },
    wnativeAbi.constructors[0].toU8a([]),
    '',
  ));

  const wnativeFactory = new Wnative_factory(api, deployer);
  const { address: wnativeAddress } = await wnativeFactory.new({
    gasLimit: gasRequired,
  });
  console.log('wnative address:', wnativeAddress);
  const wnative = new Wnative(wnativeAddress, deployer, api);

  const routerContractRaw = JSON.parse(
    fs.readFileSync(
      __dirname + `/../artifacts/router_contract.contract`,
      'utf8',
    ),
  );
  const routerAbi = new Abi(routerContractRaw);
  ({ gasRequired } = await api.call.contractsApi.instantiate(
    deployer.address,
    0,
    null,
    null,
    { Upload: routerAbi.info.source.wasm },
    routerAbi.constructors[0].toU8a([factory.address, wnative.address]),
    '',
  ));

  const routerFactory = new Router_factory(api, deployer);
  const { address: routerAddress } = await routerFactory.new(
    factory.address,
    wnative.address,
    { gasLimit: gasRequired },
  );
  console.log('router address:', routerAddress);
  const router = new Router(routerAddress, deployer, api);

  const aploAmount = parseUnits(100).toString();

  await approveSpender(aplo, router.address, aploAmount);
  console.log('approved aplo to spend by router');
  await addLiquidityNative(router, aplo, aploAmount, aploAmount, deployer.address);
  console.log('added aplo liquidity');
  await approveSpender(usdc, router.address, ONE_STABLECOIN);
  console.log('approved usdc to spend by router');
  await addLiquidityNative(router, usdc, ONE_STABLECOIN, ONE_STABLECOIN, deployer.address);
  console.log('added usdc liquidity');
  await approveSpender(usdt, router.address, ONE_STABLECOIN);
  console.log('approved usdt to spend by router');
  await addLiquidityNative(router, usdt, ONE_STABLECOIN, ONE_STABLECOIN, deployer.address);
  console.log('added usdt liquidity');

  const {
    value: { ok: aploSbyAddress },
  } = await factory.query.getPair(aplo.address, wnativeAddress);
  console.log('aploSbyAddress', aploSbyAddress);
  const {
    value: { ok: usdcSbyAddress },
  } = await factory.query.getPair(usdc.address, wnativeAddress);
  console.log('usdcSbyAddress', usdcSbyAddress);
  const {
    value: { ok: usdtSbyAddress },
  } = await factory.query.getPair(usdt.address, wnativeAddress);
  console.log('usdtSbyAddress', usdtSbyAddress);
  await api.disconnect();
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
