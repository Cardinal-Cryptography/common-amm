import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import Token_factory from '../../types/constructors/psp22';
import Token from '../../types/contracts/psp22';
import * as psp22 from './token';
import { DEADLINE } from './constants';
import { parseUnits } from './shared';
import Router from '../../types/contracts/router_contract';
import { loadAddresses } from './utils';

console.log('Setting up tokens on', process.env.WS_NODE);

const WAZERO_DECIMALS = 12;

const TOKENS_DATA = [
  { symbol: 'FIR', name: 'Fire', totalSupply: 8_000_000 },
  { symbol: 'PAP', name: 'Paper', totalSupply: 5_000_000 },
  { symbol: 'PLA', name: 'Plants', totalSupply: 1_250_000 },
  { symbol: 'WAT', name: 'Water', totalSupply: 6_250_000 },
  { symbol: 'WIN', name: 'Wind', totalSupply: 2_500_000 },
  { symbol: 'ELE', name: 'Electricity', totalSupply: 2_000_000 },
  { symbol: 'ICE', name: 'Ice', totalSupply: 4_000_000 },
  { symbol: 'STE', name: 'Steam', totalSupply: 20_000_000 },
  { symbol: 'STO', name: 'Stone', totalSupply: 12_500_000 },
  { symbol: 'WOO', name: 'Wood', totalSupply: 25_000_000 },
] as const;

const IGNORED_PAIRS = [
  ['FIR', 'WAT'],
  ['FIR', 'ICE'],
  ['WIN', 'STO'],
  ['WOO', 'ICE'],
];

const TOKEN_WITHOUT_WAZERO_PAIR = 'ICE';
const WAZERO_LIQUIDITY_IN_PAIR = 1_000_000;

const wsProvider = new WsProvider(process.env.WS_NODE);
const keyring = new Keyring({ type: 'sr25519' });

async function main(): Promise<void> {
  const api = await ApiPromise.create({ provider: wsProvider });
  const deployer = keyring.addFromUri(process.env.AUTHORITY_SEED);

  const router = new Router(loadAddresses().routerAddress, deployer, api);

  const tokenFactory = new Token_factory(api, deployer);

  const tokenInitGas = await psp22.estimateInit(api, deployer);

  const decimals = 12;
  const tokens = [];

  // deploy tokens
  for (const { symbol, name, totalSupply } of TOKENS_DATA) {
    const { address } = await tokenFactory.new(
      parseUnits(totalSupply, decimals).toString(),
      name,
      symbol,
      decimals,
      { gasLimit: tokenInitGas },
    );

    tokens.push({ symbol, name, totalSupply, address, decimals });

    console.log('Created token:', {
      symbol,
      name,
      decimals,
      address,
      totalSupply,
    });
  }

  const allTokensWithNativePair = tokens.filter(
    ({ symbol }) => symbol !== TOKEN_WITHOUT_WAZERO_PAIR,
  );

  // Add liquidity to native pairs
  for (const { name, address, totalSupply } of allTokensWithNativePair) {
    const tokenAmount = totalSupply / 10;

    const tokenAmountString = parseUnits(tokenAmount, decimals).toString();
    const wazeroAmountString = parseUnits(
      WAZERO_LIQUIDITY_IN_PAIR,
      WAZERO_DECIMALS,
    ).toString();

    const tokenContract = new Token(address, deployer, api);
    await tokenContract.tx.approve(router.address, tokenAmountString);

    const params = [
      address,
      tokenAmountString,
      tokenAmountString,
      wazeroAmountString,
      deployer.address,
      DEADLINE,
      {
        value: wazeroAmountString,
      },
    ] as const;

    try {
      await router.tx.addLiquidityNative(...params);
    } catch (e) {
      await router.query.addLiquidityNative(...params).then((r) => {
        console.error(r.value.ok.err);
      });

      throw e;
    }

    console.log('Liquidity with wAzero added for token:', {
      name,
      tokenAmount,
      WAZERO_LIQUIDITY_IN_PAIR,
    });
  }

  const allNonNativePairsWithPools = tokens
    .flatMap((token, i) =>
      tokens.slice(i + 1).map((otherToken) => [token, otherToken]),
    )
    .filter(([token0, token1]) =>
      IGNORED_PAIRS.every(
        ([ignored0, ignored1]) => token0 !== ignored0 && token1 !== ignored1,
      ),
    );

  for (const [
    { address: address0, name: name0, totalSupply: totalSupply0 },
    { address: address1, name: name1, totalSupply: totalSupply1 },
  ] of allNonNativePairsWithPools) {
    const tokenAmount0 = totalSupply0 / 10;
    const tokenAmount1 = totalSupply1 / 10;

    const tokenDrawnAmountString0 = parseUnits(
      tokenAmount0,
      decimals,
    ).toString();
    const tokenDrawnAmountString1 = parseUnits(
      tokenAmount1,
      decimals,
    ).toString();

    const tokenContract0 = new Token(address0, deployer, api);
    const tokenContract1 = new Token(address1, deployer, api);

    await tokenContract0.tx.approve(router.address, tokenDrawnAmountString0);
    await tokenContract1.tx.approve(router.address, tokenDrawnAmountString1);

    const params = [
      address0,
      address1,
      tokenDrawnAmountString0,
      tokenDrawnAmountString1,
      tokenDrawnAmountString0,
      tokenDrawnAmountString1,
      deployer.address,
      DEADLINE,
    ] as const;

    try {
      await router.tx.addLiquidity(...params);
    } catch (e) {
      await router.query.addLiquidity(...params).then((r) => {
        console.error(r.value.ok.err);
      });

      throw e;
    }

    console.log('Liquidity between tokens added:', {
      aName: name0,
      bName: name1,
      tokenAmount: tokenDrawnAmountString0,
      theOtherTokenAmount: tokenDrawnAmountString1,
    });
  }

  await api.disconnect();
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
