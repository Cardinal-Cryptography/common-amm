import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import Token from '../../types/contracts/psp22';
import Farm from '../../types/contracts/farm_contract';
import Farm_factory from '../../types/constructors/farm_contract';
import type { KeyringPair } from '@polkadot/keyring/types';
import BN from 'bn.js';
import { uploadFarm, loadAddresses, pickRandomUpToN, randomBN, estimateFarmInit, getTokenMetadata } from './utils';
import { PSP22Metadata, FarmSpec, FarmDetails, Reward } from './types';

// Create a new instance of contract
const wsProvider = new WsProvider(process.env.WS_NODE);
// Create a keyring instance
const keyring = new Keyring({ type: 'sr25519' });

// function pickRandomRewards(rewardTokens: PSP22Metadata[]): Reward[] {
//     let reward_tokens = pickRandomUpToN(rewardTokens, rewardTokens.length);
//     let returns = [];
//     for (let token of reward_tokens) {
//         let amount = randomBalance(new BN(token.my_balance));
//         returns.push({ token: token.address, amount });
//     }
//     return returns;
// }

function randomTimestampBetween(start: number, end: number): number {
    return start + Math.ceil(Math.random() * (end - start));
}

const HOUR_MILLIS = 60 * 60 * 1000;
const DAY_MILLIS = 24 * HOUR_MILLIS;
const WEEK_MILLIS = 7 * DAY_MILLIS;

function randomizeFarmSpec(pool: string, rewardTokens: PSP22Metadata[]): FarmSpec {
    const currentTimestampMillis = new Date().getTime();
    const start = randomTimestampBetween(currentTimestampMillis, currentTimestampMillis + WEEK_MILLIS);
    const end = randomTimestampBetween(start, start + 4 * WEEK_MILLIS);
    const duration = end - start;

    if (duration < 0) {
        throw new Error('Negative duration');
    }

    const randomRewards = pickRandomUpToN(rewardTokens, rewardTokens.length);
    let rewards: Reward[] = [];

    for (let token of randomRewards) {
        // Max reward rate we can affort with our balance
        const maxRewardRate = new BN(token.my_balance).div(new BN(duration));
        // Random reward rate between 0 and maxRewardRate
        if (maxRewardRate.lt(new BN(0))) {
            throw new Error('Negative max reward rate');
        }
        const amount = randomBN(new BN(duration), maxRewardRate);
        if (amount.lt(new BN(0))) {
            throw new Error('Negative amount');
        }

        if (amount.lt(new BN(duration)) && amount.gt(new BN(0))) {
            throw new Error('Amount less than duration');
        }
        rewards.push({ token: token.address, amount });
    }
    if (rewards.length === 0) {
        throw new Error('No rewards to pick from');
    }

    if (end < start) {
        throw new Error('End timestamp is before start timestamp');
    }

    return {
        poolAddress: pool,
        rewards: rewards,
        startTimestamp: start,
        endTimestamp: end,
    };
}

function printFarmSpec(farmSpec: FarmSpec): void {
    console.log('Farm spec:');
    console.log('Pool address:', farmSpec.poolAddress);
    console.log('Start timestamp:', new Date(farmSpec.startTimestamp).toUTCString());
    console.log('End timestamp:', new Date(farmSpec.endTimestamp).toUTCString());
    printRewards(farmSpec.rewards);
}

function printRewards(rewards: Reward[]): void {
    console.log('Picked ' + rewards.length + ' random rewards:')
    for (let reward of rewards) {
        console.log('Token:', reward.token, 'Amount:', reward.amount.toString());

    }
}

async function createAndStartFarm(
    api: ApiPromise,
    signer: KeyringPair,
    farm_factory: Farm_factory,
    farmSpec: FarmSpec
): Promise<FarmDetails> {
    const rewardTokensAddress = farmSpec.rewards.map((reward) => reward.token);
    const estimate = await estimateFarmInit(api, signer, [farmSpec.poolAddress, rewardTokensAddress]);
    const res = await farm_factory.new(farmSpec.poolAddress, rewardTokensAddress, { gasLimit: estimate });
    if (res.result.result?.isError) {
        console.error('Error while creating farm: ', res.result.result);
        throw new Error('Error while creating farm');
    }
    const address = res.address?.toString();
    if (address === undefined) {
        throw new Error('Farm address is undefined');
    }
    const farm = new Farm(address, signer, api);
    const rewardAmounts = farmSpec.rewards.map((reward) => reward.amount);
    for (let rewardToken of farmSpec.rewards) {
        const token = new Token(rewardToken.token, signer, api);
        await token.tx.approve(address, rewardToken.amount);
    }
    await farm.tx.ownerStartNewFarm(farmSpec.startTimestamp, farmSpec.endTimestamp, rewardAmounts);
    return { address, spec: farmSpec };
}


async function main(): Promise<void> {
    const api = await ApiPromise.create({ provider: wsProvider });
    const signer = keyring.addFromUri(process.env.AUTHORITY_SEED);

    const poolAddress = loadAddresses('./pool_addresses.json');

    let reward_tokens = [];
    for (let token of loadAddresses('./token_addresses.json')) {
        const tokenMetadata = await getTokenMetadata(api, signer, token);
        // For some tokens we don't have any balance, so we exclude them.
        if (tokenMetadata.my_balance !== '0') {
            reward_tokens.push(tokenMetadata);
        }
    }

    const _farmCodeHash = await uploadFarm(api, signer);
    const farm_factory = new Farm_factory(api, signer);

    for (let pool of poolAddress) {
        const farmSpec = randomizeFarmSpec(pool, reward_tokens);
        const farm_details = await createAndStartFarm(api, signer, farm_factory, farmSpec);
        console.log('Farm created:', farm_details);
    }
}

main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});
