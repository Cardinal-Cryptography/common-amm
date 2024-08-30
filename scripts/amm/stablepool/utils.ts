import fs from 'fs';
import { config } from 'dotenv';
import { ApiPromise } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';
import {
  ContractInstantiateResult,
  WeightV2,
} from '@polkadot/types/interfaces';
import { Abi } from '@polkadot/api-contract';

export enum PoolType {
  Stable = 'Stable',
  Rated = 'Rated',
}

export interface PoolDeploymentParams {
  poolType: PoolType;
  poolName: string;
  tokens: string[];
  rateProviders: (string | null)[] | undefined;
  decimals: number[];
  A: number;
  tradeFee: number;
  protocolFee: number;
  protocolFeeReceiver: string | null | undefined;
  owner: string | undefined;
}

/**
 * @returns Path to the file
 * @throws If file does not exist.
 */
function getPathToFile(fileName: string): string {
  const path = __dirname + '/' + fileName;
  if (!fs.existsSync(path)) {
    throw `Could not find "${fileName}" file.`;
  }
  return path;
}

/**
 * Load env file
 */
export function loadEnv() {
  const isExampleDeployment = process.argv[2] == 'example';
  const fileName = isExampleDeployment ? '.env.example' : '.env';

  config({
    path: getPathToFile(fileName),
  });
}

/**
 * Loads list of pool deployment parameters from JSON file.
 * @returns List of deployment parameters for each pool.
 */
export function loadDeploymentParams(): PoolDeploymentParams[] {
  const isExampleDeployment = process.argv[2] == 'example';
  const fileName =
    'deploymentPoolsParams' + (isExampleDeployment ? '.example.json' : '.json');
  const path = getPathToFile(fileName);

  return JSON.parse(fs.readFileSync(path).toString());
}

/**
 * Stores deployed pools in a JSON file.
 * @param pools - The pools to store.
 */
export function storeDeployedPools(
  pools: ({ address: string } & PoolDeploymentParams)[],
): void {
  let toSave = [];
  const filePath = __dirname + '/deployedPools.json';
  if (fs.existsSync(filePath)) {
    const rawData = fs.readFileSync(filePath);
    toSave = JSON.parse(rawData.toString());
  }
  fs.writeFileSync(filePath, JSON.stringify(toSave.concat(pools), null, 2));
}

/**
 * Estimates gas required to create a new instance of the StablePool contract.
 *
 * NOTE: This shouldn't be necessary but `Contract::new()` doesn't estimate gas and uses a hardcoded value.
 */
export async function estimateStablePoolInit(
  api: ApiPromise,
  deployer: KeyringPair,
  params: PoolDeploymentParams,
): Promise<WeightV2> {
  const contractRaw = JSON.parse(
    fs.readFileSync(
      __dirname + `/../../../artifacts/stable_pool_contract.contract`,
      'utf8',
    ),
  );
  const contractAbi = new Abi(contractRaw);
  let sampleArgs = [];
  let constructorId = 0;
  switch (params.poolType) {
    case PoolType.Stable:
      constructorId = 0;
      sampleArgs = [
        params.tokens,
        params.decimals,
        params.A,
        params.owner ?? deployer.address,
        params.tradeFee,
        params.protocolFee,
        params.protocolFeeReceiver,
      ];
      break;
    case PoolType.Rated:
      constructorId = 1;
      sampleArgs = [
        params.tokens,
        params.decimals,
        params.rateProviders,
        params.A,
        params.owner ?? deployer.address,
        params.tradeFee,
        params.protocolFee,
        params.protocolFeeReceiver,
      ];
      break;
  }
  const { gasRequired } = (await api.call.contractsApi.instantiate(
    deployer.address,
    0,
    null,
    null,
    { Upload: contractAbi.info.source.wasm },
    contractAbi.constructors[constructorId].toU8a(sampleArgs),
    '',
  )) as unknown as ContractInstantiateResult;
  return gasRequired;
}
