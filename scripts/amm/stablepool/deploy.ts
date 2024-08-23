import { WsProvider, Keyring, ApiPromise } from "@polkadot/api";
import StablePoolConstructors from "../../../types/constructors/stable_pool_contract";

import {
  readDeploymentParams,
  PoolType,
  storeDeployedPools,
  PoolDeploymentParams,
} from "./utils";

async function main(): Promise<void> {
  const { secrets, deployerWallet, deploymentParams } = readDeploymentParams(
    process.argv[2] == "example"
  );
  const wsProvider = new WsProvider(secrets.RPC_URL);
  const keyring = new Keyring({ type: "sr25519" });

  const api = await ApiPromise.create({ provider: wsProvider });
  const deployer = keyring.createFromJson(deployerWallet as any);
  deployer.unlock(secrets.deploymentWalletPassword);
  console.log("Using", deployer.address, "as the deployer");

  const stablePoolConstructors = new StablePoolConstructors(api, deployer);

  let deployedPools: ({ address: string } & PoolDeploymentParams)[] = [];

  for (let i = 0; i < deploymentParams.length; ++i) {
    const {
      poolType,
      tokens,
      rateProviders,
      decimals,
      A,
      tradeFee,
      protocolFee,
      protocolFeeReceiver,
      owner,
    } = deploymentParams[i];

    let address = "";
    switch (poolType) {
      case PoolType.Stable:
        address = await stablePoolConstructors
          .newStable(
            tokens,
            decimals,
            A,
            owner ?? deployer.address,
            tradeFee,
            protocolFee,
            protocolFeeReceiver
          )
          .then((res) => res.address);
        break;
      case PoolType.Rated:
        address = await stablePoolConstructors
          .newRated(
            tokens,
            decimals,
            rateProviders,
            A,
            owner ?? deployer.address,
            tradeFee,
            protocolFee,
            protocolFeeReceiver
          )
          .then((res) => res.address);
        break;
    }
    deployedPools.push({
      address,
      owner: owner ?? deployer.address,
      ...deploymentParams[i],
    });
  }

  console.log("Deployed pools:", deployedPools);

  storeDeployedPools(deployedPools);

  await api.disconnect();
  console.log("Done");
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
