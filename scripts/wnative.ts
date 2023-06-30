import { ApiPromise } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { WeightV2 } from '@polkadot/types/interfaces';
import { estimateContractInit } from './utils';

/**
 * Estimates gas required to create a new instance of `WNative` contract.
 *
 * NOTE: This shouldn't be necessary but `Contract::new()` doesn't estimate gas and uses a hardcoded value.
 */
export async function estimateInit(
  api: ApiPromise,
  deployer: KeyringPair,
): Promise<WeightV2> {
  return estimateContractInit(api, deployer, 'wnative_contract.contract', []);
}
