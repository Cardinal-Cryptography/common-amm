import type {ReturnNumber} from "@727-ventures/typechain-types";
import type * as ReturnTypes from '../types-returns/factory_contract';

export interface PairCreated {
	token0: ReturnTypes.AccountId;
	token1: ReturnTypes.AccountId;
	pair: ReturnTypes.AccountId;
	pairLen: number;
}

