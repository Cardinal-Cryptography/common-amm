import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import Token from '../../types/contracts/psp22';
import Farm from '../../types/contracts/farm_contract';
import Farm_factory from '../../types/constructors/farm_contract';
import type { KeyringPair } from '@polkadot/keyring/types';
import { uploadFarm, estimateFarmInit, loadFarmSpecs } from './utils';
import { FarmSpec, FarmDetails, Reward } from './types';

// Create a new instance of contract
const wsProvider = new WsProvider(process.env.WS_NODE);

// Create a keyring instance
const keyring = new Keyring({ type: 'sr25519' });

// Read authority seed from env var. 
const signer = keyring.addFromUri(process.env.AUTHORITY_SEED);


function printFarmSpec(farmSpec: FarmSpec): void {
    console.log('Farm spec:');
    console.log('Pool address:', farmSpec.poolAddress);
    console.log('Start timestamp:', new Date(farmSpec.startTimestamp).toUTCString());
    console.log('End timestamp:', new Date(farmSpec.endTimestamp).toUTCString());
    printRewards(farmSpec.rewards);
}

function printRewards(rewards: Reward[]): void {
    console.log('Picked ' + rewards.length + ' rewards:')
    for (let reward of rewards) {
        console.log('\tToken:', reward.token, 'Amount:', reward.amount.toString());
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
    console.log(`Farm ${address} for pool ${farmSpec.poolAddress} created successfully.`);
    const farm = new Farm(address, signer, api);
    const rewardAmounts = farmSpec.rewards.map((reward) => reward.amount);
    for (let rewardToken of farmSpec.rewards) {
        console.log('Approving token:', rewardToken.token, 'amount:', rewardToken.amount.toString());
        const token = new Token(rewardToken.token, signer, api);
        await token.tx.approve(address, rewardToken.amount);
    }
    // Check if the farm has a non-zero start and end timestamps,
    // if they're both 0, farm will not be started.
    if (farmSpec.startTimestamp != 0 && farmSpec.endTimestamp != 0) {
        const res = await farm.tx.ownerStartNewFarm(farmSpec.startTimestamp, farmSpec.endTimestamp, rewardAmounts);
        if (res.result.isError) {
            console.error('Error while starting farm: ', res.result);
            throw new Error('Error while starting farm');
        }
        console.log(`Farm ${address} started successfully.`);
    }
    return { address, spec: farmSpec };
}


async function main(): Promise<void> {
    const api = await ApiPromise.create({ provider: wsProvider });

    const farms = loadFarmSpecs("put JSON with farm spec here. eg farm_spec.json ");
    if (!farms) {
        throw new Error('No farms found');
    }
    const _farmCodeHash = await uploadFarm(api, signer);
    const farm_factory = new Farm_factory(api, signer);

    for (let farm_spec of farms) {
        printFarmSpec(farm_spec);
        await createAndStartFarm(api, signer, farm_factory, farm_spec);
    }
}

main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});
