import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import { loadAddresses, storeAddresses } from './utils';
import Token_factory from '../types/constructors/psp22_token';
import * as token from './token';
import { TOTAL_SUPPLY, STABLE_TOTAL_SUPPLY } from './constants';

// Create a new instance of contract
const wsProvider = new WsProvider(process.env.WS_NODE);
// Create a keyring instance
const keyring = new Keyring({ type: 'sr25519' });

async function main(): Promise<void> {
  const api = await ApiPromise.create({ provider: wsProvider });
  const deployer = keyring.addFromUri(process.env.AUTHORITY_SEED);
  const tokenCodeHash = await token.upload(api, deployer);
  console.log('token code hash:', tokenCodeHash);

  const addresses = loadAddresses();

  const tokenFactory = new Token_factory(api, deployer);

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
  const { address: usdcAddress } = await tokenFactory.new(
    STABLE_TOTAL_SUPPLY,
    'USD Coin',
    'USDC',
    6,
    { gasLimit: tokenInitGas },
  );
  console.log('usdc token address:', usdcAddress);
  const { address: usdtAddress } = await tokenFactory.new(
    STABLE_TOTAL_SUPPLY,
    'Tether USD',
    'USDT',
    6,
    { gasLimit: tokenInitGas },
  );
  console.log('usdt token address:', usdtAddress);

  storeAddresses({
    ...addresses,
    dogeAddress,
    usdcAddress,
    usdtAddress,
  });

  console.log('psp22 token addresses stored');

  await api.disconnect();
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
