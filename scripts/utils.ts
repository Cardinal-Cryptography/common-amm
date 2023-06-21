import Token from '../types/contracts/psp22_token';
import Router from '../types/contracts/router_contract';
import { ONE_WAZERO, DEADLINE } from './constants';

export const approveSpender = async (
  token: Token,
  spender: string,
  amount: string,
): Promise<void> => {
  await token.tx.approve(spender, amount);
};

export const addLiquidityNative = async (
  router: Router,
  token: Token,
  amountDesired: string,
  amountMin: string,
  to: string,
): Promise<void> => {
  await router.tx.addLiquidityNative(
    token.address,
    amountDesired,
    amountMin,
    ONE_WAZERO,
    to,
    DEADLINE,
    {
      value: ONE_WAZERO,
    },
  );
};
