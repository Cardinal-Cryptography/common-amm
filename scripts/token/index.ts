import fs from 'fs';
import { ApiPromise } from '@polkadot/api';
import { Abi } from '@polkadot/api-contract';
import { WeightV2 } from '@polkadot/types/interfaces';
import { TOTAL_SUPPLY } from '../constants';

/**
 * Estimates gas required to create a new instance of `PSP22_token` contract.
 *
 * NOTE: This shouldn't be necessary but `Contract::new()` doesn't estimate gas and uses a hardcoded value.
 */
export async function estimateInit(
  api: ApiPromise,
  deployer: string,
): Promise<WeightV2> {
  const tokenContractRaw = JSON.parse(
    fs.readFileSync(
      __dirname + `/../../artifacts/psp22_token.contract`,
      'utf8',
    ),
  );
  const tokenAbi = new Abi(tokenContractRaw);
  return (
    await api.call.contractsApi.instantiate(
      deployer,
      0,
      null,
      null,
      { Upload: tokenAbi.info.source.wasm },
      tokenAbi.constructors[0].toU8a([
        TOTAL_SUPPLY,
        'Apollo Token',
        'APLO',
        18,
      ]),
      '',
    )
  ).gasRequired;
}
