import Token from '../types/contracts/psp22_token';
import Router from '../types/contracts/router_contract';
import { ONE_WAZERO, DEADLINE } from './constants';
import { Addresses } from './shared';
import fs from 'fs';

import { ApiPromise } from '@polkadot/api';
import { Abi } from '@polkadot/api-contract';
import { KeyringPair } from '@polkadot/keyring/types';
import { HexString } from '@polkadot/util/types';

export const addLiquidityNative = async (
  router: Router,
  token: Token,
  amountDesired: string,
  amountMin: string,
  to: string,
): Promise<void> => {
  await router.tx.addLiquidityNative(
    token.address,
    amountDesired,
    amountMin,
    ONE_WAZERO,
    to,
    DEADLINE,
    {
      value: ONE_WAZERO,
    },
  );
};

/**
 * Stores addresses in a JSON file.
 * @param addresses - The addresses to store.
 */
export function storeAddresses(addresses: Addresses): void {
  fs.writeFileSync(
    __dirname + '/../addresses.json',
    JSON.stringify(addresses, null, 2),
  );
}

/**
 * Reads addresses from a JSON file.
 * @returns The addresses stored in the JSON file.
 */
export function loadAddresses(): Addresses {
  return JSON.parse(
    fs.readFileSync(__dirname + '/../addresses.json', 'utf8'),
  ) as Addresses;
}

/**
 * Uploads the contract to the chain.
 * @param api - The api instance.
 * @param deployer - The deployer keyring pair.
 * @param contractName - The file name of the contract to upload.
 * @returns code hash of the deployed contract.
 */
export async function uploadCode(
  api: ApiPromise,
  deployer: KeyringPair,
  contractName: string,
): Promise<HexString> {
  const tokenContractRaw = JSON.parse(
    fs.readFileSync(__dirname + `/../artifacts/` + contractName, 'utf8'),
  );
  const tokenAbi = new Abi(tokenContractRaw);
  const _txHash = await new Promise(async (resolve, reject) => {
    const unsub = await api.tx.contracts
      .uploadCode(tokenAbi.info.source.wasm, null, 0)
      .signAndSend(deployer, (result) => {
        if (result.isFinalized) {
          unsub();
          resolve(result.txHash);
        }
        if (result.isError) {
          unsub();
          reject(result);
        }
      });
  });
  return tokenAbi.info.source.wasmHash.toHex();
}
