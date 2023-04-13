/* This file is auto-generated */

import type { ContractPromise } from '@polkadot/api-contract';
import type { GasLimit, GasLimitAndRequiredValue } from '@727-ventures/typechain-types';
import { buildSubmittableExtrinsic } from '@727-ventures/typechain-types';
import type * as ArgumentTypes from '../types-arguments/pair_contract';
import type BN from 'bn.js';
import type { ApiPromise } from '@polkadot/api';



export default class Methods {
	private __nativeContract : ContractPromise;
	private __apiPromise: ApiPromise;

	constructor(
		nativeContract : ContractPromise,
		apiPromise: ApiPromise,
	) {
		this.__nativeContract = nativeContract;
		this.__apiPromise = apiPromise;
	}
	/**
	 * allowance
	 *
	 * @param { ArgumentTypes.AccountId } owner,
	 * @param { ArgumentTypes.AccountId } spender,
	*/
	"allowance" (
		owner: ArgumentTypes.AccountId,
		spender: ArgumentTypes.AccountId,
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "psp22::allowance", [owner, spender], __options);
	}

	/**
	 * transferFrom
	 *
	 * @param { ArgumentTypes.AccountId } from,
	 * @param { ArgumentTypes.AccountId } to,
	 * @param { (string | number | BN) } value,
	 * @param { Array<(number | string | BN)> } data,
	*/
	"transferFrom" (
		from: ArgumentTypes.AccountId,
		to: ArgumentTypes.AccountId,
		value: (string | number | BN),
		data: Array<(number | string | BN)>,
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "psp22::transferFrom", [from, to, value, data], __options);
	}

	/**
	 * transfer
	 *
	 * @param { ArgumentTypes.AccountId } to,
	 * @param { (string | number | BN) } value,
	 * @param { Array<(number | string | BN)> } data,
	*/
	"transfer" (
		to: ArgumentTypes.AccountId,
		value: (string | number | BN),
		data: Array<(number | string | BN)>,
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "psp22::transfer", [to, value, data], __options);
	}

	/**
	 * totalSupply
	 *
	*/
	"totalSupply" (
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "psp22::totalSupply", [], __options);
	}

	/**
	 * approve
	 *
	 * @param { ArgumentTypes.AccountId } spender,
	 * @param { (string | number | BN) } value,
	*/
	"approve" (
		spender: ArgumentTypes.AccountId,
		value: (string | number | BN),
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "psp22::approve", [spender, value], __options);
	}

	/**
	 * balanceOf
	 *
	 * @param { ArgumentTypes.AccountId } owner,
	*/
	"balanceOf" (
		owner: ArgumentTypes.AccountId,
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "psp22::balanceOf", [owner], __options);
	}

	/**
	 * decreaseAllowance
	 *
	 * @param { ArgumentTypes.AccountId } spender,
	 * @param { (string | number | BN) } deltaValue,
	*/
	"decreaseAllowance" (
		spender: ArgumentTypes.AccountId,
		deltaValue: (string | number | BN),
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "psp22::decreaseAllowance", [spender, deltaValue], __options);
	}

	/**
	 * increaseAllowance
	 *
	 * @param { ArgumentTypes.AccountId } spender,
	 * @param { (string | number | BN) } deltaValue,
	*/
	"increaseAllowance" (
		spender: ArgumentTypes.AccountId,
		deltaValue: (string | number | BN),
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "psp22::increaseAllowance", [spender, deltaValue], __options);
	}

	/**
	 * transferOwnership
	 *
	 * @param { ArgumentTypes.AccountId } newOwner,
	*/
	"transferOwnership" (
		newOwner: ArgumentTypes.AccountId,
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "ownable::transferOwnership", [newOwner], __options);
	}

	/**
	 * owner
	 *
	*/
	"owner" (
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "ownable::owner", [], __options);
	}

	/**
	 * renounceOwnership
	 *
	*/
	"renounceOwnership" (
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "ownable::renounceOwnership", [], __options);
	}

	/**
	 * getToken0
	 *
	*/
	"getToken0" (
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "pair::getToken0", [], __options);
	}

	/**
	 * getToken1
	 *
	*/
	"getToken1" (
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "pair::getToken1", [], __options);
	}

	/**
	 * price1CumulativeLast
	 *
	*/
	"price1CumulativeLast" (
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "pair::price1CumulativeLast", [], __options);
	}

	/**
	 * initialize
	 *
	 * @param { ArgumentTypes.AccountId } token0,
	 * @param { ArgumentTypes.AccountId } token1,
	*/
	"initialize" (
		token0: ArgumentTypes.AccountId,
		token1: ArgumentTypes.AccountId,
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "pair::initialize", [token0, token1], __options);
	}

	/**
	 * getReserves
	 *
	*/
	"getReserves" (
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "pair::getReserves", [], __options);
	}

	/**
	 * burn
	 *
	 * @param { ArgumentTypes.AccountId } to,
	*/
	"burn" (
		to: ArgumentTypes.AccountId,
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "pair::burn", [to], __options);
	}

	/**
	 * swap
	 *
	 * @param { (string | number | BN) } amount0Out,
	 * @param { (string | number | BN) } amount1Out,
	 * @param { ArgumentTypes.AccountId } to,
	*/
	"swap" (
		amount0Out: (string | number | BN),
		amount1Out: (string | number | BN),
		to: ArgumentTypes.AccountId,
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "pair::swap", [amount0Out, amount1Out, to], __options);
	}

	/**
	 * skim
	 *
	 * @param { ArgumentTypes.AccountId } to,
	*/
	"skim" (
		to: ArgumentTypes.AccountId,
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "pair::skim", [to], __options);
	}

	/**
	 * sync
	 *
	*/
	"sync" (
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "pair::sync", [], __options);
	}

	/**
	 * mint
	 *
	 * @param { ArgumentTypes.AccountId } to,
	*/
	"mint" (
		to: ArgumentTypes.AccountId,
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "pair::mint", [to], __options);
	}

	/**
	 * price0CumulativeLast
	 *
	*/
	"price0CumulativeLast" (
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "pair::price0CumulativeLast", [], __options);
	}

}