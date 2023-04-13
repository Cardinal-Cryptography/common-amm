/* This file is auto-generated */

import type { ContractPromise } from '@polkadot/api-contract';
import type { ApiPromise } from '@polkadot/api';
import type { GasLimit, GasLimitAndRequiredValue, Result } from '@727-ventures/typechain-types';
import type { QueryReturnType } from '@727-ventures/typechain-types';
import { queryJSON, queryOkJSON, handleReturnType } from '@727-ventures/typechain-types';
import type * as ArgumentTypes from '../types-arguments/router_contract';
import type * as ReturnTypes from '../types-returns/router_contract';
import type BN from 'bn.js';
//@ts-ignore
import {ReturnNumber} from '@727-ventures/typechain-types';
import {getTypeDescription} from './../shared/utils';
import DATA_TYPE_DESCRIPTIONS from '../data/router_contract.json';


export default class Methods {
	private __nativeContract : ContractPromise;
	private __apiPromise: ApiPromise;
	private __callerAddress : string;

	constructor(
		nativeContract : ContractPromise,
		nativeApi : ApiPromise,
		callerAddress : string,
	) {
		this.__nativeContract = nativeContract;
		this.__callerAddress = callerAddress;
		this.__apiPromise = nativeApi;
	}

	/**
	* getAmountsOut
	*
	* @param { (string | number | BN) } amountIn,
	* @param { Array<ArgumentTypes.AccountId> } path,
	* @returns { Result<Result<Array<ReturnNumber>, ReturnTypes.RouterError>, ReturnTypes.LangError> }
	*/
	"getAmountsOut" (
		amountIn: (string | number | BN),
		path: Array<ArgumentTypes.AccountId>,
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<Array<ReturnNumber>, ReturnTypes.RouterError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "router::getAmountsOut", [amountIn, path], __options , (result) => { return handleReturnType(result, getTypeDescription(8, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* getAmountIn
	*
	* @param { (string | number | BN) } amountOut,
	* @param { (string | number | BN) } reserveIn,
	* @param { (string | number | BN) } reserveOut,
	* @returns { Result<Result<ReturnNumber, ReturnTypes.RouterError>, ReturnTypes.LangError> }
	*/
	"getAmountIn" (
		amountOut: (string | number | BN),
		reserveIn: (string | number | BN),
		reserveOut: (string | number | BN),
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<ReturnNumber, ReturnTypes.RouterError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "router::getAmountIn", [amountOut, reserveIn, reserveOut], __options , (result) => { return handleReturnType(result, getTypeDescription(20, DATA_TYPE_DESCRIPTIONS)); });
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
	* @returns { Result<Result<[ReturnNumber, ReturnNumber], ReturnTypes.RouterError>, ReturnTypes.LangError> }
	*/
	"removeLiquidity" (
		tokenA: ArgumentTypes.AccountId,
		tokenB: ArgumentTypes.AccountId,
		liquidity: (string | number | BN),
		amountAMin: (string | number | BN),
		amountBMin: (string | number | BN),
		to: ArgumentTypes.AccountId,
		deadline: (number | string | BN),
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<[ReturnNumber, ReturnNumber], ReturnTypes.RouterError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "router::removeLiquidity", [tokenA, tokenB, liquidity, amountAMin, amountBMin, to, deadline], __options , (result) => { return handleReturnType(result, getTypeDescription(23, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* swapExactNativeForTokens
	*
	* @param { (string | number | BN) } amountOutMin,
	* @param { Array<ArgumentTypes.AccountId> } path,
	* @param { ArgumentTypes.AccountId } to,
	* @param { (number | string | BN) } deadline,
	* @returns { Result<Result<Array<ReturnNumber>, ReturnTypes.RouterError>, ReturnTypes.LangError> }
	*/
	"swapExactNativeForTokens" (
		amountOutMin: (string | number | BN),
		path: Array<ArgumentTypes.AccountId>,
		to: ArgumentTypes.AccountId,
		deadline: (number | string | BN),
		__options ? : GasLimitAndRequiredValue,
	): Promise< QueryReturnType< Result<Result<Array<ReturnNumber>, ReturnTypes.RouterError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "router::swapExactNativeForTokens", [amountOutMin, path, to, deadline], __options , (result) => { return handleReturnType(result, getTypeDescription(8, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* wnative
	*
	* @returns { Result<ReturnTypes.AccountId, ReturnTypes.LangError> }
	*/
	"wnative" (
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<ReturnTypes.AccountId, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "router::wnative", [], __options , (result) => { return handleReturnType(result, getTypeDescription(26, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* quote
	*
	* @param { (string | number | BN) } amountA,
	* @param { (string | number | BN) } reserveA,
	* @param { (string | number | BN) } reserveB,
	* @returns { Result<Result<ReturnNumber, ReturnTypes.RouterError>, ReturnTypes.LangError> }
	*/
	"quote" (
		amountA: (string | number | BN),
		reserveA: (string | number | BN),
		reserveB: (string | number | BN),
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<ReturnNumber, ReturnTypes.RouterError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "router::quote", [amountA, reserveA, reserveB], __options , (result) => { return handleReturnType(result, getTypeDescription(20, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* swapTokensForExactNative
	*
	* @param { (string | number | BN) } amountOut,
	* @param { (string | number | BN) } amountInMax,
	* @param { Array<ArgumentTypes.AccountId> } path,
	* @param { ArgumentTypes.AccountId } to,
	* @param { (number | string | BN) } deadline,
	* @returns { Result<Result<Array<ReturnNumber>, ReturnTypes.RouterError>, ReturnTypes.LangError> }
	*/
	"swapTokensForExactNative" (
		amountOut: (string | number | BN),
		amountInMax: (string | number | BN),
		path: Array<ArgumentTypes.AccountId>,
		to: ArgumentTypes.AccountId,
		deadline: (number | string | BN),
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<Array<ReturnNumber>, ReturnTypes.RouterError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "router::swapTokensForExactNative", [amountOut, amountInMax, path, to, deadline], __options , (result) => { return handleReturnType(result, getTypeDescription(8, DATA_TYPE_DESCRIPTIONS)); });
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
	* @returns { Result<Result<[ReturnNumber, ReturnNumber, ReturnNumber], ReturnTypes.RouterError>, ReturnTypes.LangError> }
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
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<[ReturnNumber, ReturnNumber, ReturnNumber], ReturnTypes.RouterError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "router::addLiquidity", [tokenA, tokenB, amountADesired, amountBDesired, amountAMin, amountBMin, to, deadline], __options , (result) => { return handleReturnType(result, getTypeDescription(27, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* swapTokensForExactTokens
	*
	* @param { (string | number | BN) } amountOut,
	* @param { (string | number | BN) } amountInMax,
	* @param { Array<ArgumentTypes.AccountId> } path,
	* @param { ArgumentTypes.AccountId } to,
	* @param { (number | string | BN) } deadline,
	* @returns { Result<Result<Array<ReturnNumber>, ReturnTypes.RouterError>, ReturnTypes.LangError> }
	*/
	"swapTokensForExactTokens" (
		amountOut: (string | number | BN),
		amountInMax: (string | number | BN),
		path: Array<ArgumentTypes.AccountId>,
		to: ArgumentTypes.AccountId,
		deadline: (number | string | BN),
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<Array<ReturnNumber>, ReturnTypes.RouterError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "router::swapTokensForExactTokens", [amountOut, amountInMax, path, to, deadline], __options , (result) => { return handleReturnType(result, getTypeDescription(8, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* getAmountsIn
	*
	* @param { (string | number | BN) } amountOut,
	* @param { Array<ArgumentTypes.AccountId> } path,
	* @returns { Result<Result<Array<ReturnNumber>, ReturnTypes.RouterError>, ReturnTypes.LangError> }
	*/
	"getAmountsIn" (
		amountOut: (string | number | BN),
		path: Array<ArgumentTypes.AccountId>,
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<Array<ReturnNumber>, ReturnTypes.RouterError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "router::getAmountsIn", [amountOut, path], __options , (result) => { return handleReturnType(result, getTypeDescription(8, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* swapNativeForExactTokens
	*
	* @param { (string | number | BN) } amountOut,
	* @param { Array<ArgumentTypes.AccountId> } path,
	* @param { ArgumentTypes.AccountId } to,
	* @param { (number | string | BN) } deadline,
	* @returns { Result<Result<Array<ReturnNumber>, ReturnTypes.RouterError>, ReturnTypes.LangError> }
	*/
	"swapNativeForExactTokens" (
		amountOut: (string | number | BN),
		path: Array<ArgumentTypes.AccountId>,
		to: ArgumentTypes.AccountId,
		deadline: (number | string | BN),
		__options ? : GasLimitAndRequiredValue,
	): Promise< QueryReturnType< Result<Result<Array<ReturnNumber>, ReturnTypes.RouterError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "router::swapNativeForExactTokens", [amountOut, path, to, deadline], __options , (result) => { return handleReturnType(result, getTypeDescription(8, DATA_TYPE_DESCRIPTIONS)); });
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
	* @returns { Result<Result<[ReturnNumber, ReturnNumber, ReturnNumber], ReturnTypes.RouterError>, ReturnTypes.LangError> }
	*/
	"addLiquidityNative" (
		token: ArgumentTypes.AccountId,
		amountTokenDesired: (string | number | BN),
		amountTokenMin: (string | number | BN),
		amountNativeMin: (string | number | BN),
		to: ArgumentTypes.AccountId,
		deadline: (number | string | BN),
		__options ? : GasLimitAndRequiredValue,
	): Promise< QueryReturnType< Result<Result<[ReturnNumber, ReturnNumber, ReturnNumber], ReturnTypes.RouterError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "router::addLiquidityNative", [token, amountTokenDesired, amountTokenMin, amountNativeMin, to, deadline], __options , (result) => { return handleReturnType(result, getTypeDescription(27, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* getAmountOut
	*
	* @param { (string | number | BN) } amountIn,
	* @param { (string | number | BN) } reserveIn,
	* @param { (string | number | BN) } reserveOut,
	* @returns { Result<Result<ReturnNumber, ReturnTypes.RouterError>, ReturnTypes.LangError> }
	*/
	"getAmountOut" (
		amountIn: (string | number | BN),
		reserveIn: (string | number | BN),
		reserveOut: (string | number | BN),
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<ReturnNumber, ReturnTypes.RouterError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "router::getAmountOut", [amountIn, reserveIn, reserveOut], __options , (result) => { return handleReturnType(result, getTypeDescription(20, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* factory
	*
	* @returns { Result<ReturnTypes.AccountId, ReturnTypes.LangError> }
	*/
	"factory" (
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<ReturnTypes.AccountId, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "router::factory", [], __options , (result) => { return handleReturnType(result, getTypeDescription(26, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* swapExactTokensForNative
	*
	* @param { (string | number | BN) } amountIn,
	* @param { (string | number | BN) } amountOutMin,
	* @param { Array<ArgumentTypes.AccountId> } path,
	* @param { ArgumentTypes.AccountId } to,
	* @param { (number | string | BN) } deadline,
	* @returns { Result<Result<Array<ReturnNumber>, ReturnTypes.RouterError>, ReturnTypes.LangError> }
	*/
	"swapExactTokensForNative" (
		amountIn: (string | number | BN),
		amountOutMin: (string | number | BN),
		path: Array<ArgumentTypes.AccountId>,
		to: ArgumentTypes.AccountId,
		deadline: (number | string | BN),
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<Array<ReturnNumber>, ReturnTypes.RouterError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "router::swapExactTokensForNative", [amountIn, amountOutMin, path, to, deadline], __options , (result) => { return handleReturnType(result, getTypeDescription(8, DATA_TYPE_DESCRIPTIONS)); });
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
	* @returns { Result<Result<[ReturnNumber, ReturnNumber], ReturnTypes.RouterError>, ReturnTypes.LangError> }
	*/
	"removeLiquidityNative" (
		token: ArgumentTypes.AccountId,
		liquidity: (string | number | BN),
		amountTokenMin: (string | number | BN),
		amountNativeMin: (string | number | BN),
		to: ArgumentTypes.AccountId,
		deadline: (number | string | BN),
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<[ReturnNumber, ReturnNumber], ReturnTypes.RouterError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "router::removeLiquidityNative", [token, liquidity, amountTokenMin, amountNativeMin, to, deadline], __options , (result) => { return handleReturnType(result, getTypeDescription(23, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* swapExactTokensForTokens
	*
	* @param { (string | number | BN) } amountIn,
	* @param { (string | number | BN) } amountOutMin,
	* @param { Array<ArgumentTypes.AccountId> } path,
	* @param { ArgumentTypes.AccountId } to,
	* @param { (number | string | BN) } deadline,
	* @returns { Result<Result<Array<ReturnNumber>, ReturnTypes.RouterError>, ReturnTypes.LangError> }
	*/
	"swapExactTokensForTokens" (
		amountIn: (string | number | BN),
		amountOutMin: (string | number | BN),
		path: Array<ArgumentTypes.AccountId>,
		to: ArgumentTypes.AccountId,
		deadline: (number | string | BN),
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<Array<ReturnNumber>, ReturnTypes.RouterError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "router::swapExactTokensForTokens", [amountIn, amountOutMin, path, to, deadline], __options , (result) => { return handleReturnType(result, getTypeDescription(8, DATA_TYPE_DESCRIPTIONS)); });
	}

}