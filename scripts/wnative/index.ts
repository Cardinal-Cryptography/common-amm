import fs from 'fs';
import { ApiPromise } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { WeightV2 } from '@polkadot/types/interfaces';
import { Abi } from '@polkadot/api-contract';

/**
 * Estimates gas required to create a new instance of `WNative` contract.
 *
 * NOTE: This shouldn't be necessary but `Contract::new()` doesn't estimate gas and uses a hardcoded value.
 */
export async function estimateInit(
  api: ApiPromise,
  deployer: KeyringPair,
): Promise<WeightV2> {
  const wnativeContractRaw = JSON.parse(
    fs.readFileSync(
      __dirname + `/../../artifacts/wnative_contract.contract`,
      'utf8',
    ),
  );
  const wnativeAbi = new Abi(wnativeContractRaw);
  const { gasRequired } = await api.call.contractsApi.instantiate(
    deployer.address,
    0,
    null,
    null,
    { Upload: wnativeAbi.info.source.wasm },
    wnativeAbi.constructors[0].toU8a([]),
    '',
  );
  return gasRequired;
}
