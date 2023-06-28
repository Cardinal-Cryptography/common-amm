import { ApiPromise } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { HexString } from '@polkadot/util/types';
import { uploadCode } from '../utils';

/**
 * Uploads the `pair_contract` contract to the chain.
 * @param api
 * @param deployer
 * @returns code hash of the deployed contract.
 */
export async function upload(
  api: ApiPromise,
  deployer: KeyringPair,
): Promise<HexString> {
  return uploadCode(api, deployer, 'pair_contract.contract');
}
