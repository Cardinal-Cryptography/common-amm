import type {ReturnNumber} from "@727-ventures/typechain-types";
import type * as ReturnTypes from '../types-returns/pair_contract';

export interface Mint {
	sender: ReturnTypes.AccountId;
	amount0: ReturnNumber;
	amount1: ReturnNumber;
}

export interface Burn {
	sender: ReturnTypes.AccountId;
	amount0: ReturnNumber;
	amount1: ReturnNumber;
	to: ReturnTypes.AccountId;
}

export interface Swap {
	sender: ReturnTypes.AccountId;
	amount0In: ReturnNumber;
	amount1In: ReturnNumber;
	amount0Out: ReturnNumber;
	amount1Out: ReturnNumber;
	to: ReturnTypes.AccountId;
}

export interface Sync {
	reserve0: ReturnNumber;
	reserve1: ReturnNumber;
}

export interface Transfer {
	from: ReturnTypes.AccountId | null;
	to: ReturnTypes.AccountId | null;
	value: ReturnNumber;
}

export interface Approval {
	owner: ReturnTypes.AccountId;
	spender: ReturnTypes.AccountId;
	value: ReturnNumber;
}

