import fs from 'fs';
import { ApiPromise } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { WeightV2 } from '@polkadot/types/interfaces';
import { Abi } from '@polkadot/api-contract';
import { DUMMY_ADDRESS } from '../constants';

/**
 * Estimates gas required to create a new instance of `Router` contract.
 *
 * NOTE: This shouldn't be necessary but `Contract::new()` doesn't estimate gas and uses a hardcoded value.
 */
export async function estimateInit(
  api: ApiPromise,
  deployer: KeyringPair,
): Promise<WeightV2> {
  const routerContractRaw = JSON.parse(
    fs.readFileSync(
      __dirname + `/../../artifacts/router_contract.contract`,
      'utf8',
    ),
  );
  const routerAbi = new Abi(routerContractRaw);
  const { gasRequired } = await api.call.contractsApi.instantiate(
    deployer.address,
    0,
    null,
    null,
    { Upload: routerAbi.info.source.wasm },
    routerAbi.constructors[0].toU8a([DUMMY_ADDRESS, DUMMY_ADDRESS]),
    '',
  );
  return gasRequired;
}
