import fs from "fs";
import { KeyringPair$Json } from "@polkadot/keyring/types";

export enum PoolType {
  Stable = "Stable",
  Rated = "Rated",
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

export interface Secrets {
  deploymentWalletPassword: string;
  RPC_URL: string;
}

export function readDeploymentParams(example?: boolean): {
  secrets: Secrets;
  deployerWallet: KeyringPair$Json;
  deploymentParams: PoolDeploymentParams[];
} {
  let secrets: Secrets,
    deployerWallet: KeyringPair$Json,
    deploymentParams: PoolDeploymentParams[];

  const extension = example ? ".example.json" : ".json";

  secrets = JSON.parse(
    fs.readFileSync(__dirname + "/secrets" + extension).toString()
  );

  deployerWallet = JSON.parse(
    fs.readFileSync(__dirname + "/deployerWallet" + extension).toString()
  );

  deploymentParams = JSON.parse(
    fs.readFileSync(__dirname + "/deploymentPoolsParams" + extension).toString()
  );

  return { secrets, deployerWallet, deploymentParams };
}

/**
 * Stores deployed pools in a JSON file.
 * @param pools - The pools to store.
 */
export function storeDeployedPools(
  pools: ({ address: string } & PoolDeploymentParams)[]
): void {
  let toSave = [];
  const filePath = __dirname + "/deployedPools.json";
  if (fs.existsSync(filePath)) {
    const rawData = fs.readFileSync(filePath);
    toSave = JSON.parse(rawData.toString());
  }
  fs.writeFileSync(filePath, JSON.stringify(toSave.concat(pools), null, 2));
}
