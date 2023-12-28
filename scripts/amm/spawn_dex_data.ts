import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import Token_factory from '../../types/constructors/psp22';
import Token from '../../types/contracts/psp22'
import * as psp22 from './token';
import {DEADLINE} from './constants';
import {parseUnits} from './shared';
import Router from '../../types/contracts/router_contract';
import {loadAddresses} from './utils';
import BN from 'bn.js';

console.log('Setting up tokens on', process.env.WS_NODE)

const WAZERO_DECIMALS = 12
const POSSIBLE_DECIMALS = [10, 12, 15, 18]
const NUMBER_OF_DIFFERENT_TOTAL_TOKEN_SUPPLY_LEVELS = 8
const TOKENS_DATA = [
  { symbol: 'CRC', name: 'Circle', },
  { symbol: 'TGL', name: 'Triangle', },
  { symbol: 'QUA', name: 'Quadrangle', },
  { symbol: 'PEN', name: 'Pentagon', },
  { symbol: 'HGO', name: 'Hexagon', },
  { symbol: 'HTG', name: 'Heptagon', },
  { symbol: 'OGO', name: 'Octagon', },
  { symbol: 'NOGO', name: 'Nonagon', },
  { symbol: 'DEGO', name: 'Decagon', },
  { symbol: 'UGO', name: 'Undecagon', },
] as const

const wsProvider = new WsProvider(process.env.WS_NODE);
const keyring = new Keyring({ type: 'sr25519' });

async function main(): Promise<void> {
  const api = await ApiPromise.create({ provider: wsProvider });
  const deployer = keyring.addFromUri(process.env.AUTHORITY_SEED);
  const psp22ContractCodeHash = await psp22.upload(api, deployer);
  console.log('PSP22 contract code hash:', psp22ContractCodeHash);

  const router = new Router(loadAddresses().routerAddress, deployer, api);

  const tokenFactory = new Token_factory(api, deployer);

  const tokenInitGas = await psp22.estimateInit(api, deployer);

  const tokens = await runPromisesInSeries(TOKENS_DATA.map(({ symbol, name }, i) => async () => {
    const decimals = POSSIBLE_DECIMALS.at(i % POSSIBLE_DECIMALS.length);
    const minTotalSupply = 10_000
    const maxTotalSupply = 999_999_999
    const totalSupply = Math.round((
      (i + NUMBER_OF_DIFFERENT_TOTAL_TOKEN_SUPPLY_LEVELS) * (maxTotalSupply - minTotalSupply) / NUMBER_OF_DIFFERENT_TOTAL_TOKEN_SUPPLY_LEVELS
    ) % (maxTotalSupply - minTotalSupply) + minTotalSupply)

    const { address } = await tokenFactory.new(
      parseUnits(totalSupply, decimals).toString(),
      name,
      symbol,
      decimals,
      { gasLimit: tokenInitGas },
    );

    console.log('Created token:', {
      symbol,
      name,
      decimals,
      address,
      totalSupply
    })

    return { symbol, name, address, decimals }
  }))

  await runPromisesInSeries(tokens.map(({ address, decimals, name }, i) => async () => {
    console.log('Starting processing token', name)

    const hasLiquidityWithWazero = !!((i + 1) % 3)

    if (hasLiquidityWithWazero) {
      console.log('Starting adding liquidity with wAzero')

      const tokenAmount = (i % 4 + 1) * 1000
      const wazeroAmount = ((i + 2) % 4 + 1) * 1000

      const tokenAmountString = parseUnits(tokenAmount, decimals).toString()
      const wazeroAmountString = parseUnits(wazeroAmount, WAZERO_DECIMALS).toString()

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
      ] as const

      try {
        await router.tx.addLiquidityNative(...params);
      } catch (e) {
        await router.query.addLiquidityNative(...params).then(r => {
          console.error(r.value.ok.err)
        })

        throw e
      }

      console.log('Liquidity with wAzero added for token:', {
        name,
        tokenAmount,
        wazeroAmount
      })
    } else {
      console.log('Liquidity with wAzero not added for token:', { name })
    }

    await runPromisesInSeries(tokens.slice(i + 1).map((theOtherToken, j) => async () => {
      console.log('Starting processing a pair with', theOtherToken.name)

      const hasLiquidityWithTheOtherToken = !!((j + 1) % 4)

      if (hasLiquidityWithTheOtherToken) {
        console.log('Starting adding liquidity with', theOtherToken.name)

        const tokenDrawnAmount = ((i + j + 3) % 4 + 1) * 1000
        const theOtherTokenDrawnAmount = ((Math.abs(i - j) + 4) % 4 + 1) * 1000

        const tokenDrawnAmountString = parseUnits(tokenDrawnAmount, decimals).toString()
        const theOtherTokenDrawnAmountString = parseUnits(theOtherTokenDrawnAmount, theOtherToken.decimals).toString()

        const tokenContract = new Token(address, deployer, api);
        const theOtherTokenContract = new Token(theOtherToken.address, deployer, api);

        const tokenAmount = BN.max(
          BN.min(
            new BN(tokenDrawnAmountString),
            await tokenContract.query.balanceOf(deployer.address)
              .then(res => res.value.ok?.rawNumber)
          ),
          new BN(0)
        )
        const theOtherTokenAmount = BN.max(
          BN.min(
            new BN(theOtherTokenDrawnAmountString),
            await theOtherTokenContract.query.balanceOf(deployer.address)
              .then(res => res.value.ok?.rawNumber)
          ),
          new BN(0)
        )

        if (tokenAmount.lten(0) || theOtherTokenAmount.lten(0)) {
          console.log('Liquidity between tokens not added due to lack of balance of one of them:', {
            aName: name,
            bName: theOtherToken.name,
          })
          return
        }

        await tokenContract.tx.approve(router.address, tokenAmount);
        await theOtherTokenContract.tx.approve(router.address, theOtherTokenAmount);

        const params = [
          address,
          theOtherToken.address,
          tokenAmount,
          theOtherTokenAmount,
          tokenAmount,
          theOtherTokenAmount,
          deployer.address,
          DEADLINE,
        ] as const

        try {
          await router.tx.addLiquidity(...params);
        } catch (e) {
          await router.query.addLiquidity(...params).then(r => {
            console.error(r.value.ok.err)
          });

          throw e
        }

        console.log('Liquidity between tokens added:', {
          aName: name,
          bName: theOtherToken.name,
          tokenAmount: Number(tokenAmount.toString().slice(0, -decimals)),
          theOtherTokenAmount: Number(theOtherTokenAmount.toString().slice(0, -theOtherToken.decimals))
        })
      } else {
        console.log('Liquidity between tokens not added:', {
          aName: name,
          bName: theOtherToken.name,
        })
      }
    }))
  }))

  await api.disconnect();
}

const runPromisesInSeries = async <T>(tasks: (() => Promise<T>)[]) => {
  const results: T[] = []

  await tasks.reduce((queue, executeTask) =>
    queue.then(() => executeTask().then(result => results.push(result))),
    Promise.resolve()
  )

  return results
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
