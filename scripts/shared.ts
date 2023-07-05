export function parseUnits(amount: bigint | number, decimals): bigint {
  return BigInt(amount) * 10n ** BigInt(decimals);
}

export type Addresses = {
  pairCodeHash: string;
  factoryAddress: string;
  wnativeAddress: string;
  routerAddress: string;
};
