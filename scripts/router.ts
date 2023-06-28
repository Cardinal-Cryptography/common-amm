import { ApiPromise } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { WeightV2 } from '@polkadot/types/interfaces';
import { DUMMY_ADDRESS } from './constants';
import { estimateContractInit } from './utils';

/**
 * Estimates gas required to create a new instance of `Router` contract.
 *
 * NOTE: This shouldn't be necessary but `Contract::new()` doesn't estimate gas and uses a hardcoded value.
 */
export async function estimateInit(
  api: ApiPromise,
  deployer: KeyringPair,
): Promise<WeightV2> {
  return estimateContractInit(api, deployer, 'router_contract.contract', [
    DUMMY_ADDRESS,
    DUMMY_ADDRESS,
  ]);
}
