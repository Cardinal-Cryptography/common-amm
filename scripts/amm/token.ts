import { ApiPromise } from '@polkadot/api';
import { WeightV2 } from '@polkadot/types/interfaces';
import { KeyringPair } from '@polkadot/keyring/types';
import { HexString } from '@polkadot/util/types';
import { TOTAL_SUPPLY } from './constants';
import { uploadCode, estimateContractInit } from './utils';

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
  deployer: KeyringPair,
): Promise<WeightV2> {
  return estimateContractInit(api, deployer, 'psp22_token.contract', [
    TOTAL_SUPPLY,
    'Doge Coin',
    'DOGE',
    18,
  ]);
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
