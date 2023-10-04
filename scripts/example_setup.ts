import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import { loadAddresses } from './utils';
import Token_factory from '../types/constructors/psp22_token';
import Token from '../types/contracts/psp22_token';
import Router from '../types/contracts/router_contract';
import Factory from '../types/contracts/factory_contract';
import * as token from './token';
import { TOTAL_SUPPLY, STABLE_TOTAL_SUPPLY, ONE_THOUSAND_STABLECOIN } from './constants';
import { addLiquidityNative } from './utils';
import { parseUnits } from './shared';

// Create a new instance of contract
const wsProvider = new WsProvider(process.env.WS_NODE);
// Create a keyring instance
const keyring = new Keyring({ type: 'sr25519' });

async function main(): Promise<void> {
  const api = await ApiPromise.create({ provider: wsProvider });
  const deployer = keyring.addFromUri(process.env.AUTHORITY_SEED);
  const tokenCodeHash = await token.upload(api, deployer);
  console.log('token code hash:', tokenCodeHash);

  const { routerAddress, factoryAddress, wnativeAddress } = loadAddresses();

  const tokenFactory = new Token_factory(api, deployer);
  const router = new Router(routerAddress, deployer, api);
  const factory = new Factory(factoryAddress, deployer, api);

  /// Create tokens

  const tokenInitGas = await token.estimateInit(api, deployer);
  const { address: dogeAddress } = await tokenFactory.new(
    TOTAL_SUPPLY,
    'Doge Coin',
    'DOGE',
    18,
    { gasLimit: tokenInitGas },
  );
  console.log('doge coin address', dogeAddress);
  const doge = new Token(dogeAddress, deployer, api);
  const { address: usdcAddress } = await tokenFactory.new(
    STABLE_TOTAL_SUPPLY,
    'USD Coin',
    'USDC',
    6,
    { gasLimit: tokenInitGas },
  );
  console.log('usdc token address:', usdcAddress);
  const usdc = new Token(usdcAddress, deployer, api);
  const { address: usdtAddress } = await tokenFactory.new(
    STABLE_TOTAL_SUPPLY,
    'Tether USD',
    'USDT',
    6,
    { gasLimit: tokenInitGas },
  );
  console.log('usdt token address:', usdtAddress);
  const usdt = new Token(usdtAddress, deployer, api);

  /// Add liquidity
  const dogeAmount = parseUnits(100, 18).toString();

  await doge.tx.approve(router.address, dogeAmount);
  console.log('approved doge to spend by router');
  await addLiquidityNative(
    router,
    doge,
    dogeAmount,
    dogeAmount,
    deployer.address,
  );
  console.log('added doge liquidity');
  await usdc.tx.approve(router.address, ONE_THOUSAND_STABLECOIN);
  console.log('approved usdc to spend by router');
  await addLiquidityNative(
    router,
    usdc,
    ONE_THOUSAND_STABLECOIN,
    ONE_THOUSAND_STABLECOIN,
    deployer.address,
  );
  console.log('added usdc liquidity');
  await usdt.tx.approve(router.address, ONE_THOUSAND_STABLECOIN);
  console.log('approved usdt to spend by router');
  await addLiquidityNative(
    router,
    usdt,
    ONE_THOUSAND_STABLECOIN,
    ONE_THOUSAND_STABLECOIN,
    deployer.address,
  );
  console.log('added usdt liquidity');

  /// Query pair addresses
  const {
    value: { ok: dogeWAzeroAddress },
  } = await factory.query.getPair(doge.address, wnativeAddress);
  console.log('dogeWAzeroAddress', dogeWAzeroAddress);
  const {
    value: { ok: usdcWAzeroAddress },
  } = await factory.query.getPair(usdc.address, wnativeAddress);
  console.log('usdcWAzeroAddress', usdcWAzeroAddress);
  const {
    value: { ok: usdtWAzeroAddress },
  } = await factory.query.getPair(usdt.address, wnativeAddress);
  console.log('usdtWAzeroAddress', usdtWAzeroAddress);

  await api.disconnect();
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
