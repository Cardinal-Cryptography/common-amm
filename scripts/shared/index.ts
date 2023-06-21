export function parseUnits(amount: bigint | number, decimals = 18): bigint {
  return BigInt(amount) * 10n ** BigInt(decimals);
}

export type Addresses = {
  pairCodeHash: string;
  tokenCodeHash: string;
  factoryAddress: string;
  wnativeAddress: string;
  routerAddress: string;
};
