import {CodePromise} from "@polkadot/api-contract";
import type {KeyringPair} from "@polkadot/keyring/types";
import type {ApiPromise} from "@polkadot/api";
import {_genValidGasLimitAndValue, _signAndSend, SignAndSendSuccessResponse} from "@727-ventures/typechain-types";
import type {ConstructorOptions} from "@727-ventures/typechain-types";
import type {WeightV2} from "@polkadot/types/interfaces";
import type * as ArgumentTypes from '../types-arguments/router_contract';
import { ContractFile } from '../contract-info/router_contract';
import BN from 'bn.js';

export default class Constructors {
	readonly nativeAPI: ApiPromise;
	readonly signer: KeyringPair;

	constructor(
		nativeAPI: ApiPromise,
		signer: KeyringPair,
	) {
		this.nativeAPI = nativeAPI;
		this.signer = signer;
	}

	/**
	* new
	*
	* @param { ArgumentTypes.AccountId } factory,
	* @param { ArgumentTypes.AccountId } wnative,
	*/
   	async "new" (
		factory: ArgumentTypes.AccountId,
		wnative: ArgumentTypes.AccountId,
		__options ? : ConstructorOptions,
   	) {
   		const __contract = JSON.parse(ContractFile);
		const code = new CodePromise(this.nativeAPI, __contract, __contract.source.wasm);

		let newGasLimit = this.nativeAPI.registry.createType('WeightV2', {
			refTime: new BN(11_113_433_193),
			proofSize: new BN(19_456),
		  }) as unknown as WeightV2;

		const storageDepositLimit = __options?.storageDepositLimit;
			const tx = code.tx["new"]!({ gasLimit: newGasLimit, storageDepositLimit, value: __options?.value }, factory, wnative);
			let response;

			try {
				response = await _signAndSend(this.nativeAPI.registry, tx, this.signer, (event: any) => event);
			}
			catch (error) {
				console.log(error);
			}

		return {
			result: response as SignAndSendSuccessResponse,
			// @ts-ignore
			address: (response as SignAndSendSuccessResponse)!.result!.contract.address.toString(),
		};
	}
}