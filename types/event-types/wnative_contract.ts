import type {ReturnNumber} from "@727-ventures/typechain-types";
import type * as ReturnTypes from '../types-returns/wnative_contract';

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

