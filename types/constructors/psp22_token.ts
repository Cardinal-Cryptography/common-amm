import {CodePromise} from "@polkadot/api-contract";
import type {KeyringPair} from "@polkadot/keyring/types";
import type {ApiPromise} from "@polkadot/api";
import {_genValidGasLimitAndValue, _signAndSend, SignAndSendSuccessResponse} from "@727-ventures/typechain-types";
import type {ConstructorOptions} from "@727-ventures/typechain-types";
import type {WeightV2} from "@polkadot/types/interfaces";
import type * as ArgumentTypes from '../types-arguments/psp22_token';
import { ContractFile } from '../contract-info/psp22_token';
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
	* @param { (string | number | BN) } totalSupply,
	* @param { Array<(number | string | BN)> | null } name,
	* @param { Array<(number | string | BN)> | null } symbol,
	* @param { (number | string | BN) } decimals,
	*/
   	async "new" (
		totalSupply: (string | number | BN),
		name: Array<(number | string | BN)> | null,
		symbol: Array<(number | string | BN)> | null,
		decimals: (number | string | BN),
		__options ? : ConstructorOptions,
   	) {
   		const __contract = JSON.parse(ContractFile);
		const code = new CodePromise(this.nativeAPI, __contract, __contract.source.wasm);

		let newGasLimit = this.nativeAPI.registry.createType('WeightV2', {
			refTime: new BN(11_113_433_193),
			proofSize: new BN(19_456),
		  }) as unknown as WeightV2;

		const storageDepositLimit = __options?.storageDepositLimit;
			const tx = code.tx["new"]!({ gasLimit: newGasLimit, storageDepositLimit, value: __options?.value }, totalSupply, name, symbol, decimals);
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