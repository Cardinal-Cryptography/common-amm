import fs from 'fs';
import { config } from 'dotenv';

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
