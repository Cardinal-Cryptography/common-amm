import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import { loadAddresses } from './utils';
import Token from '../../types/contracts/psp22_token';
import Router from '../../types/contracts/router_contract';
import Factory from '../../types/contracts/factory_contract';
import { ONE_THOUSAND_STABLECOIN } from './constants';
import { addLiquidityNative } from './utils';
import { parseUnits } from './shared';

// Create a new instance of contract
const wsProvider = new WsProvider(process.env.WS_NODE);
// Create a keyring instance
const keyring = new Keyring({ type: 'sr25519' });

async function main(): Promise<void> {
  const api = await ApiPromise.create({ provider: wsProvider });
  const deployer = keyring.addFromUri(process.env.AUTHORITY_SEED);

  const {
    routerAddress,
    factoryAddress,
    wnativeAddress,
    dogeAddress,
    usdcAddress,
    usdtAddress,
  } = loadAddresses();

  const router = new Router(routerAddress, deployer, api);
  const factory = new Factory(factoryAddress, deployer, api);

  /// Create tokens

  const doge = new Token(dogeAddress, deployer, api);
  const usdc = new Token(usdcAddress, deployer, api);
  const usdt = new Token(usdtAddress, deployer, api);

  /// Add liquidity
  const dogeAmount = parseUnits(1000, 18).toString();

  await doge.tx.approve(router.address, dogeAmount);
  console.log('approved 1000 DOGE to spend by router');
  await addLiquidityNative(
    router,
    doge,
    dogeAmount,
    dogeAmount,
    deployer.address,
  );
  console.log('added 1000 DOGE liquidity');
  await usdc.tx.approve(router.address, ONE_THOUSAND_STABLECOIN);
  console.log('approved 1000 USDC to spend by router');
  await addLiquidityNative(
    router,
    usdc,
    ONE_THOUSAND_STABLECOIN,
    ONE_THOUSAND_STABLECOIN,
    deployer.address,
  );
  console.log('added 1000 USDC liquidity');
  await usdt.tx.approve(router.address, ONE_THOUSAND_STABLECOIN);
  console.log('approved 1000 USDT to spend by router');
  await addLiquidityNative(
    router,
    usdt,
    ONE_THOUSAND_STABLECOIN,
    ONE_THOUSAND_STABLECOIN,
    deployer.address,
  );
  console.log('added 1000 USDT liquidity');

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
