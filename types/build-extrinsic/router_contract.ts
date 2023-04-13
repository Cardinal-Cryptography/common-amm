/* This file is auto-generated */

import type { ContractPromise } from '@polkadot/api-contract';
import type { GasLimit, GasLimitAndRequiredValue } from '@727-ventures/typechain-types';
import { buildSubmittableExtrinsic } from '@727-ventures/typechain-types';
import type * as ArgumentTypes from '../types-arguments/router_contract';
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
	 * getAmountsOut
	 *
	 * @param { (string | number | BN) } amountIn,
	 * @param { Array<ArgumentTypes.AccountId> } path,
	*/
	"getAmountsOut" (
		amountIn: (string | number | BN),
		path: Array<ArgumentTypes.AccountId>,
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "router::getAmountsOut", [amountIn, path], __options);
	}

	/**
	 * getAmountIn
	 *
	 * @param { (string | number | BN) } amountOut,
	 * @param { (string | number | BN) } reserveIn,
	 * @param { (string | number | BN) } reserveOut,
	*/
	"getAmountIn" (
		amountOut: (string | number | BN),
		reserveIn: (string | number | BN),
		reserveOut: (string | number | BN),
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "router::getAmountIn", [amountOut, reserveIn, reserveOut], __options);
	}

	/**
	 * removeLiquidity
	 *
	 * @param { ArgumentTypes.AccountId } tokenA,
	 * @param { ArgumentTypes.AccountId } tokenB,
	 * @param { (string | number | BN) } liquidity,
	 * @param { (string | number | BN) } amountAMin,
	 * @param { (string | number | BN) } amountBMin,
	 * @param { ArgumentTypes.AccountId } to,
	 * @param { (number | string | BN) } deadline,
	*/
	"removeLiquidity" (
		tokenA: ArgumentTypes.AccountId,
		tokenB: ArgumentTypes.AccountId,
		liquidity: (string | number | BN),
		amountAMin: (string | number | BN),
		amountBMin: (string | number | BN),
		to: ArgumentTypes.AccountId,
		deadline: (number | string | BN),
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "router::removeLiquidity", [tokenA, tokenB, liquidity, amountAMin, amountBMin, to, deadline], __options);
	}

	/**
	 * swapExactNativeForTokens
	 *
	 * @param { (string | number | BN) } amountOutMin,
	 * @param { Array<ArgumentTypes.AccountId> } path,
	 * @param { ArgumentTypes.AccountId } to,
	 * @param { (number | string | BN) } deadline,
	*/
	"swapExactNativeForTokens" (
		amountOutMin: (string | number | BN),
		path: Array<ArgumentTypes.AccountId>,
		to: ArgumentTypes.AccountId,
		deadline: (number | string | BN),
		__options: GasLimitAndRequiredValue,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "router::swapExactNativeForTokens", [amountOutMin, path, to, deadline], __options);
	}

	/**
	 * wnative
	 *
	*/
	"wnative" (
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "router::wnative", [], __options);
	}

	/**
	 * quote
	 *
	 * @param { (string | number | BN) } amountA,
	 * @param { (string | number | BN) } reserveA,
	 * @param { (string | number | BN) } reserveB,
	*/
	"quote" (
		amountA: (string | number | BN),
		reserveA: (string | number | BN),
		reserveB: (string | number | BN),
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "router::quote", [amountA, reserveA, reserveB], __options);
	}

	/**
	 * swapTokensForExactNative
	 *
	 * @param { (string | number | BN) } amountOut,
	 * @param { (string | number | BN) } amountInMax,
	 * @param { Array<ArgumentTypes.AccountId> } path,
	 * @param { ArgumentTypes.AccountId } to,
	 * @param { (number | string | BN) } deadline,
	*/
	"swapTokensForExactNative" (
		amountOut: (string | number | BN),
		amountInMax: (string | number | BN),
		path: Array<ArgumentTypes.AccountId>,
		to: ArgumentTypes.AccountId,
		deadline: (number | string | BN),
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "router::swapTokensForExactNative", [amountOut, amountInMax, path, to, deadline], __options);
	}

	/**
	 * addLiquidity
	 *
	 * @param { ArgumentTypes.AccountId } tokenA,
	 * @param { ArgumentTypes.AccountId } tokenB,
	 * @param { (string | number | BN) } amountADesired,
	 * @param { (string | number | BN) } amountBDesired,
	 * @param { (string | number | BN) } amountAMin,
	 * @param { (string | number | BN) } amountBMin,
	 * @param { ArgumentTypes.AccountId } to,
	 * @param { (number | string | BN) } deadline,
	*/
	"addLiquidity" (
		tokenA: ArgumentTypes.AccountId,
		tokenB: ArgumentTypes.AccountId,
		amountADesired: (string | number | BN),
		amountBDesired: (string | number | BN),
		amountAMin: (string | number | BN),
		amountBMin: (string | number | BN),
		to: ArgumentTypes.AccountId,
		deadline: (number | string | BN),
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "router::addLiquidity", [tokenA, tokenB, amountADesired, amountBDesired, amountAMin, amountBMin, to, deadline], __options);
	}

	/**
	 * swapTokensForExactTokens
	 *
	 * @param { (string | number | BN) } amountOut,
	 * @param { (string | number | BN) } amountInMax,
	 * @param { Array<ArgumentTypes.AccountId> } path,
	 * @param { ArgumentTypes.AccountId } to,
	 * @param { (number | string | BN) } deadline,
	*/
	"swapTokensForExactTokens" (
		amountOut: (string | number | BN),
		amountInMax: (string | number | BN),
		path: Array<ArgumentTypes.AccountId>,
		to: ArgumentTypes.AccountId,
		deadline: (number | string | BN),
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "router::swapTokensForExactTokens", [amountOut, amountInMax, path, to, deadline], __options);
	}

	/**
	 * getAmountsIn
	 *
	 * @param { (string | number | BN) } amountOut,
	 * @param { Array<ArgumentTypes.AccountId> } path,
	*/
	"getAmountsIn" (
		amountOut: (string | number | BN),
		path: Array<ArgumentTypes.AccountId>,
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "router::getAmountsIn", [amountOut, path], __options);
	}

	/**
	 * swapNativeForExactTokens
	 *
	 * @param { (string | number | BN) } amountOut,
	 * @param { Array<ArgumentTypes.AccountId> } path,
	 * @param { ArgumentTypes.AccountId } to,
	 * @param { (number | string | BN) } deadline,
	*/
	"swapNativeForExactTokens" (
		amountOut: (string | number | BN),
		path: Array<ArgumentTypes.AccountId>,
		to: ArgumentTypes.AccountId,
		deadline: (number | string | BN),
		__options: GasLimitAndRequiredValue,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "router::swapNativeForExactTokens", [amountOut, path, to, deadline], __options);
	}

	/**
	 * addLiquidityNative
	 *
	 * @param { ArgumentTypes.AccountId } token,
	 * @param { (string | number | BN) } amountTokenDesired,
	 * @param { (string | number | BN) } amountTokenMin,
	 * @param { (string | number | BN) } amountNativeMin,
	 * @param { ArgumentTypes.AccountId } to,
	 * @param { (number | string | BN) } deadline,
	*/
	"addLiquidityNative" (
		token: ArgumentTypes.AccountId,
		amountTokenDesired: (string | number | BN),
		amountTokenMin: (string | number | BN),
		amountNativeMin: (string | number | BN),
		to: ArgumentTypes.AccountId,
		deadline: (number | string | BN),
		__options: GasLimitAndRequiredValue,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "router::addLiquidityNative", [token, amountTokenDesired, amountTokenMin, amountNativeMin, to, deadline], __options);
	}

	/**
	 * getAmountOut
	 *
	 * @param { (string | number | BN) } amountIn,
	 * @param { (string | number | BN) } reserveIn,
	 * @param { (string | number | BN) } reserveOut,
	*/
	"getAmountOut" (
		amountIn: (string | number | BN),
		reserveIn: (string | number | BN),
		reserveOut: (string | number | BN),
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "router::getAmountOut", [amountIn, reserveIn, reserveOut], __options);
	}

	/**
	 * factory
	 *
	*/
	"factory" (
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "router::factory", [], __options);
	}

	/**
	 * swapExactTokensForNative
	 *
	 * @param { (string | number | BN) } amountIn,
	 * @param { (string | number | BN) } amountOutMin,
	 * @param { Array<ArgumentTypes.AccountId> } path,
	 * @param { ArgumentTypes.AccountId } to,
	 * @param { (number | string | BN) } deadline,
	*/
	"swapExactTokensForNative" (
		amountIn: (string | number | BN),
		amountOutMin: (string | number | BN),
		path: Array<ArgumentTypes.AccountId>,
		to: ArgumentTypes.AccountId,
		deadline: (number | string | BN),
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "router::swapExactTokensForNative", [amountIn, amountOutMin, path, to, deadline], __options);
	}

	/**
	 * removeLiquidityNative
	 *
	 * @param { ArgumentTypes.AccountId } token,
	 * @param { (string | number | BN) } liquidity,
	 * @param { (string | number | BN) } amountTokenMin,
	 * @param { (string | number | BN) } amountNativeMin,
	 * @param { ArgumentTypes.AccountId } to,
	 * @param { (number | string | BN) } deadline,
	*/
	"removeLiquidityNative" (
		token: ArgumentTypes.AccountId,
		liquidity: (string | number | BN),
		amountTokenMin: (string | number | BN),
		amountNativeMin: (string | number | BN),
		to: ArgumentTypes.AccountId,
		deadline: (number | string | BN),
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "router::removeLiquidityNative", [token, liquidity, amountTokenMin, amountNativeMin, to, deadline], __options);
	}

	/**
	 * swapExactTokensForTokens
	 *
	 * @param { (string | number | BN) } amountIn,
	 * @param { (string | number | BN) } amountOutMin,
	 * @param { Array<ArgumentTypes.AccountId> } path,
	 * @param { ArgumentTypes.AccountId } to,
	 * @param { (number | string | BN) } deadline,
	*/
	"swapExactTokensForTokens" (
		amountIn: (string | number | BN),
		amountOutMin: (string | number | BN),
		path: Array<ArgumentTypes.AccountId>,
		to: ArgumentTypes.AccountId,
		deadline: (number | string | BN),
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "router::swapExactTokensForTokens", [amountIn, amountOutMin, path, to, deadline], __options);
	}

}