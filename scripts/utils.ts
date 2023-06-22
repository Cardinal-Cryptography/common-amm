import Token from '../types/contracts/psp22_token';
import Router from '../types/contracts/router_contract';
import { ONE_WAZERO, DEADLINE } from './constants';
import { Addresses } from './shared';
import fs from 'fs';

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

/**
 * Stores addresses in a JSON file.
 * @param addresses - The addresses to store.
 */
export function storeAddresses(addresses: Addresses): void {
  fs.writeFileSync(
    __dirname + '/../addresses.json',
    JSON.stringify(addresses, null, 2),
  );
}

/**
 * Reads addresses from a JSON file.
 * @returns The addresses stored in the JSON file.
 */
export function loadAddresses(): Addresses {
  return JSON.parse(
    fs.readFileSync(__dirname + '/../addresses.json', 'utf8'),
  ) as Addresses;
}
