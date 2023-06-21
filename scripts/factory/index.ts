import fs from 'fs';
import { ApiPromise } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { WeightV2 } from "@polkadot/types/interfaces";
import { Abi } from '@polkadot/api-contract';
import { DUMMY_ADDRESS } from '../constants';

/** 
 * Estimates gas required to create a new instance of `Factory` contract.
 * 
 * NOTE: This shouldn't be necessary but `Contract::new()` doesn't estimate gas and uses a hardcoded value.
  */
export async function estimateInit(api: ApiPromise, deployer: KeyringPair): Promise<WeightV2> {
  const factoryContractRaw = JSON.parse(
    fs.readFileSync(
      __dirname + `/../../artifacts/factory_contract.contract`,
      'utf8',
    ),
  );
  const factoryAbi = new Abi(factoryContractRaw);
  let { gasRequired } = await api.call.contractsApi.instantiate(
    deployer.address,
    0,
    null,
    null,
    { Upload: factoryAbi.info.source.wasm },
    factoryAbi.constructors[0].toU8a([deployer.address, DUMMY_ADDRESS]),
    '',
  );
  return gasRequired
}