import fs from 'fs';
import { ApiPromise } from '@polkadot/api';
import { Abi } from '@polkadot/api-contract';
import {
  WeightV2,
  ContractInstantiateResult,
} from '@polkadot/types/interfaces';
import { KeyringPair } from '@polkadot/keyring/types';
import { HexString } from '@polkadot/util/types';
import { TOTAL_SUPPLY } from './constants';
import { uploadCode } from './utils';

/**
 * Estimates gas required to create a new instance of `PSP22_token` contract.
 *
 * NOTE: This shouldn't be necessary but `Contract::new()` doesn't estimate gas and uses a hardcoded value.
 * @param api
 * @param deployer
 * @returns
 */
export async function estimateInit(
  api: ApiPromise,
  deployer: string,
): Promise<WeightV2> {
  const tokenContractRaw = JSON.parse(
    fs.readFileSync(__dirname + `/../artifacts/psp22_token.contract`, 'utf8'),
  );
  const tokenAbi = new Abi(tokenContractRaw);
  const { gasRequired } = (await api.call.contractsApi.instantiate(
    deployer,
    0,
    null,
    null,
    { Upload: tokenAbi.info.source.wasm },
    tokenAbi.constructors[0].toU8a([TOTAL_SUPPLY, 'Apollo Token', 'APLO', 18]),
    '',
  )) as unknown as ContractInstantiateResult;

  return gasRequired;
}

/**
 * Uploads the `PSP22_token` contract to the chain.
 * @param api
 * @param deployer
 * @returns code hash of the deployed contract.
 */
export async function upload(
  api: ApiPromise,
  deployer: KeyringPair,
): Promise<HexString> {
  return uploadCode(api, deployer, 'psp22_token.contract');
}
