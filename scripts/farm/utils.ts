import Token from '../../types/contracts/psp22';
import fs from 'fs';
import { PSP22Metadata } from './types';

import { ApiPromise } from '@polkadot/api';
import { Abi } from '@polkadot/api-contract';
import { KeyringPair } from '@polkadot/keyring/types';
import { HexString } from '@polkadot/util/types';
import {
    ContractInstantiateResult,
    WeightV2,
} from '@polkadot/types/interfaces';

export function loadAddresses(filePath: string): string[] {
    // Read the content of the file
    const fileContent = fs.readFileSync(filePath, 'utf-8');

    // Parse the JSON array from the file content
    const jsonArray: string[] = JSON.parse(fileContent);

    // Make sure the parsed content is an array of strings
    if (
        Array.isArray(jsonArray) &&
        jsonArray.every((item) => typeof item === 'string')
    ) {
        return jsonArray;
    } else {
        throw new Error(
            'Invalid JSON format. The file should contain a JSON array of strings.',
        );
    }
}

export async function getTokenMetadata(
    api: ApiPromise,
    signer: KeyringPair,
    tokenAddress: string,
): Promise<PSP22Metadata> {
    const token = new Token(tokenAddress, signer, api);
    const {
        value: { ok: name },
    } = await token.query.tokenName();
    const {
        value: { ok: symbol },
    } = await token.query.tokenSymbol();
    const {
        value: { ok: decimals },
    } = await token.query.tokenDecimals();

    const {
        value: { ok: total_supply },
    } = await token.query.totalSupply();

    const {
        value: { ok: my_balance },
    } = await token.query.balanceOf(signer.address);

    return {
        address: tokenAddress,
        name,
        symbol,
        decimals,
        total_supply: total_supply.toString(),
        my_balance: my_balance.toString(),
    } as PSP22Metadata;
}

export async function uploadFarm(api: ApiPromise, deployer: KeyringPair): Promise<HexString> {
    return uploadCode(api, deployer, 'farm_contract.contract');
}

/**
 * Uploads the contract to the chain.
 * @param api - The api instance.
 * @param deployer - The deployer keyring pair.
 * @param contractName - The file name of the contract to upload.
 * @returns code hash of the deployed contract.
 */
export async function uploadCode(
    api: ApiPromise,
    deployer: KeyringPair,
    contractName: string,
): Promise<HexString> {
    const tokenContractRaw = JSON.parse(
        fs.readFileSync(__dirname + `/../../artifacts/` + contractName, 'utf8'),
    );
    const tokenAbi = new Abi(tokenContractRaw);
    const _txHash = await new Promise(async (resolve, reject) => {
        const unsub = await api.tx.contracts
            .uploadCode(tokenAbi.info.source.wasm, null, 0)
            .signAndSend(deployer, (result) => {
                if (result.isFinalized) {
                    unsub();
                    resolve(result.txHash);
                }
                if (result.isError) {
                    unsub();
                    reject(result);
                }
            });
    });
    return tokenAbi.info.source.wasmHash.toHex();
}

/**
 * Estimates gas required to create a new instance of the contract.
 *
 * NOTE: This shouldn't be necessary but `Contract::new()` doesn't estimate gas and uses a hardcoded value.
 */
export async function estimateContractInit(
    api: ApiPromise,
    deployer: KeyringPair,
    contractName: string,
    sampleArgs: unknown[],
): Promise<WeightV2> {
    const contractRaw = JSON.parse(
        fs.readFileSync(__dirname + `/../../artifacts/` + contractName, 'utf8'),
    );
    const contractAbi = new Abi(contractRaw);
    const { gasRequired } = (await api.call.contractsApi.instantiate(
        deployer.address,
        0,
        null,
        null,
        { Upload: contractAbi.info.source.wasm },
        contractAbi.constructors[0].toU8a(sampleArgs),
        '',
    )) as unknown as ContractInstantiateResult;
    return gasRequired;
}

export function pickRandomUpToN<T>(arr: T[], n: number): T[] {
    return pickRandomN(arr, Math.ceil(Math.random() * n));
}

export function pickRandomN<T>(arr: T[], n: number): T[] {
    const result = new Array(n);
    let len = arr.length;
    const taken = new Array(len);
    if (n > len)
        throw new RangeError('getRandom: more elements taken than available');
    while (n--) {
        const x = Math.floor(Math.random() * len);
        result[n] = arr[x in taken ? taken[x] : x];
        taken[x] = --len in taken ? taken[len] : len;
    }
    return result;
}

import BN from 'bn.js';

/// Returns a random balance between min and max and occassionally 0.
export function randomBN(min: BN, max: BN): BN {
    const base_rates = [max.div(new BN(100_000)), max.div(new BN(5_000)), max.div(new BN(10_000)), max.div(new BN(1_000)), max.div(new BN(500))];
    let shifted_by_min = base_rates.map((rate) => rate.add(min));
    /// Throw in 0 sometimes
    shifted_by_min.push(new BN(0), new BN(0));
    return pickRandomN(shifted_by_min, 1)[0];
}

export function estimateFarmInit(
    api: ApiPromise,
    deployer: KeyringPair,
    sampleArgs: unknown[],
): Promise<WeightV2> {
    return estimateContractInit(api, deployer, 'farm_contract.contract', sampleArgs);
}