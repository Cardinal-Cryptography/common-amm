import type BN from 'bn.js';

export type AccountId = string | number[]

export type Hash = string | number[]

export enum LangError {
	couldNotReadInput = 'CouldNotReadInput'
}

export interface FactoryError {
	pairError ? : PairError,
	callerIsNotFeeSetter ? : null,
	zeroAddress ? : null,
	identicalAddresses ? : null,
	pairExists ? : null,
	pairInstantiationFailed ? : null
}

export class FactoryErrorBuilder {
	static PairError(value: PairError): FactoryError {
		return {
			pairError: value,
		};
	}
	static CallerIsNotFeeSetter(): FactoryError {
		return {
			callerIsNotFeeSetter: null,
		};
	}
	static ZeroAddress(): FactoryError {
		return {
			zeroAddress: null,
		};
	}
	static IdenticalAddresses(): FactoryError {
		return {
			identicalAddresses: null,
		};
	}
	static PairExists(): FactoryError {
		return {
			pairExists: null,
		};
	}
	static PairInstantiationFailed(): FactoryError {
		return {
			pairInstantiationFailed: null,
		};
	}
}

export interface PairError {
	psp22Error ? : PSP22Error,
	ownableError ? : OwnableError,
	reentrancyGuardError ? : ReentrancyGuardError,
	langError ? : LangError,
	transferError ? : null,
	k ? : null,
	insufficientLiquidityMinted ? : null,
	insufficientLiquidityBurned ? : null,
	insufficientOutputAmount ? : null,
	insufficientLiquidity ? : null,
	insufficientInputAmount ? : null,
	safeTransferFailed ? : null,
	invalidTo ? : null,
	overflow ? : null,
	locked ? : null,
	subUnderFlow1 ? : null,
	subUnderFlow2 ? : null,
	subUnderFlow3 ? : null,
	subUnderFlow4 ? : null,
	subUnderFlow5 ? : null,
	subUnderFlow6 ? : null,
	subUnderFlow7 ? : null,
	subUnderFlow8 ? : null,
	subUnderFlow9 ? : null,
	subUnderFlow10 ? : null,
	subUnderFlow11 ? : null,
	subUnderFlow12 ? : null,
	subUnderFlow13 ? : null,
	subUnderFlow14 ? : null,
	mulOverFlow1 ? : null,
	mulOverFlow2 ? : null,
	mulOverFlow3 ? : null,
	mulOverFlow4 ? : null,
	mulOverFlow5 ? : null,
	mulOverFlow6 ? : null,
	mulOverFlow7 ? : null,
	mulOverFlow8 ? : null,
	mulOverFlow9 ? : null,
	mulOverFlow10 ? : null,
	mulOverFlow11 ? : null,
	mulOverFlow12 ? : null,
	mulOverFlow13 ? : null,
	mulOverFlow14 ? : null,
	divByZero1 ? : null,
	divByZero2 ? : null,
	divByZero3 ? : null,
	divByZero4 ? : null,
	divByZero5 ? : null,
	addOverflow1 ? : null,
	castOverflow1 ? : null,
	castOverflow2 ? : null
}

export class PairErrorBuilder {
	static PSP22Error(value: PSP22Error): PairError {
		return {
			psp22Error: value,
		};
	}
	static OwnableError(value: OwnableError): PairError {
		return {
			ownableError: value,
		};
	}
	static ReentrancyGuardError(value: ReentrancyGuardError): PairError {
		return {
			reentrancyGuardError: value,
		};
	}
	static LangError(value: LangError): PairError {
		return {
			langError: value,
		};
	}
	static TransferError(): PairError {
		return {
			transferError: null,
		};
	}
	static K(): PairError {
		return {
			k: null,
		};
	}
	static InsufficientLiquidityMinted(): PairError {
		return {
			insufficientLiquidityMinted: null,
		};
	}
	static InsufficientLiquidityBurned(): PairError {
		return {
			insufficientLiquidityBurned: null,
		};
	}
	static InsufficientOutputAmount(): PairError {
		return {
			insufficientOutputAmount: null,
		};
	}
	static InsufficientLiquidity(): PairError {
		return {
			insufficientLiquidity: null,
		};
	}
	static InsufficientInputAmount(): PairError {
		return {
			insufficientInputAmount: null,
		};
	}
	static SafeTransferFailed(): PairError {
		return {
			safeTransferFailed: null,
		};
	}
	static InvalidTo(): PairError {
		return {
			invalidTo: null,
		};
	}
	static Overflow(): PairError {
		return {
			overflow: null,
		};
	}
	static Locked(): PairError {
		return {
			locked: null,
		};
	}
	static SubUnderFlow1(): PairError {
		return {
			subUnderFlow1: null,
		};
	}
	static SubUnderFlow2(): PairError {
		return {
			subUnderFlow2: null,
		};
	}
	static SubUnderFlow3(): PairError {
		return {
			subUnderFlow3: null,
		};
	}
	static SubUnderFlow4(): PairError {
		return {
			subUnderFlow4: null,
		};
	}
	static SubUnderFlow5(): PairError {
		return {
			subUnderFlow5: null,
		};
	}
	static SubUnderFlow6(): PairError {
		return {
			subUnderFlow6: null,
		};
	}
	static SubUnderFlow7(): PairError {
		return {
			subUnderFlow7: null,
		};
	}
	static SubUnderFlow8(): PairError {
		return {
			subUnderFlow8: null,
		};
	}
	static SubUnderFlow9(): PairError {
		return {
			subUnderFlow9: null,
		};
	}
	static SubUnderFlow10(): PairError {
		return {
			subUnderFlow10: null,
		};
	}
	static SubUnderFlow11(): PairError {
		return {
			subUnderFlow11: null,
		};
	}
	static SubUnderFlow12(): PairError {
		return {
			subUnderFlow12: null,
		};
	}
	static SubUnderFlow13(): PairError {
		return {
			subUnderFlow13: null,
		};
	}
	static SubUnderFlow14(): PairError {
		return {
			subUnderFlow14: null,
		};
	}
	static MulOverFlow1(): PairError {
		return {
			mulOverFlow1: null,
		};
	}
	static MulOverFlow2(): PairError {
		return {
			mulOverFlow2: null,
		};
	}
	static MulOverFlow3(): PairError {
		return {
			mulOverFlow3: null,
		};
	}
	static MulOverFlow4(): PairError {
		return {
			mulOverFlow4: null,
		};
	}
	static MulOverFlow5(): PairError {
		return {
			mulOverFlow5: null,
		};
	}
	static MulOverFlow6(): PairError {
		return {
			mulOverFlow6: null,
		};
	}
	static MulOverFlow7(): PairError {
		return {
			mulOverFlow7: null,
		};
	}
	static MulOverFlow8(): PairError {
		return {
			mulOverFlow8: null,
		};
	}
	static MulOverFlow9(): PairError {
		return {
			mulOverFlow9: null,
		};
	}
	static MulOverFlow10(): PairError {
		return {
			mulOverFlow10: null,
		};
	}
	static MulOverFlow11(): PairError {
		return {
			mulOverFlow11: null,
		};
	}
	static MulOverFlow12(): PairError {
		return {
			mulOverFlow12: null,
		};
	}
	static MulOverFlow13(): PairError {
		return {
			mulOverFlow13: null,
		};
	}
	static MulOverFlow14(): PairError {
		return {
			mulOverFlow14: null,
		};
	}
	static DivByZero1(): PairError {
		return {
			divByZero1: null,
		};
	}
	static DivByZero2(): PairError {
		return {
			divByZero2: null,
		};
	}
	static DivByZero3(): PairError {
		return {
			divByZero3: null,
		};
	}
	static DivByZero4(): PairError {
		return {
			divByZero4: null,
		};
	}
	static DivByZero5(): PairError {
		return {
			divByZero5: null,
		};
	}
	static AddOverflow1(): PairError {
		return {
			addOverflow1: null,
		};
	}
	static CastOverflow1(): PairError {
		return {
			castOverflow1: null,
		};
	}
	static CastOverflow2(): PairError {
		return {
			castOverflow2: null,
		};
	}
}

export interface PSP22Error {
	custom ? : Array<(number | string | BN)>,
	insufficientBalance ? : null,
	insufficientAllowance ? : null,
	zeroRecipientAddress ? : null,
	zeroSenderAddress ? : null,
	safeTransferCheckFailed ? : Array<(number | string | BN)>
}

export class PSP22ErrorBuilder {
	static Custom(value: Array<(number | string | BN)>): PSP22Error {
		return {
			custom: value,
		};
	}
	static InsufficientBalance(): PSP22Error {
		return {
			insufficientBalance: null,
		};
	}
	static InsufficientAllowance(): PSP22Error {
		return {
			insufficientAllowance: null,
		};
	}
	static ZeroRecipientAddress(): PSP22Error {
		return {
			zeroRecipientAddress: null,
		};
	}
	static ZeroSenderAddress(): PSP22Error {
		return {
			zeroSenderAddress: null,
		};
	}
	static SafeTransferCheckFailed(value: Array<(number | string | BN)>): PSP22Error {
		return {
			safeTransferCheckFailed: value,
		};
	}
}

export enum OwnableError {
	callerIsNotOwner = 'CallerIsNotOwner',
	newOwnerIsZero = 'NewOwnerIsZero'
}

export enum ReentrancyGuardError {
	reentrantCall = 'ReentrantCall'
}

