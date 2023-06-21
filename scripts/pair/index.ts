import fs from 'fs';
import { ApiPromise } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { HexString } from '@polkadot/util/types';
import { Abi } from '@polkadot/api-contract';

export async function upload(
  api: ApiPromise,
  deployer: KeyringPair,
): Promise<HexString> {
  const pairContractRaw = JSON.parse(
    fs.readFileSync(
      __dirname + `/../../artifacts/pair_contract.contract`,
      'utf8',
    ),
  );
  const pairAbi = new Abi(pairContractRaw);
  await new Promise(async (resolve, reject) => {
    const unsub = await api.tx.contracts
      .uploadCode(pairAbi.info.source.wasm, null, 0)
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
  return pairAbi.info.source.wasmHash.toHex();
}
