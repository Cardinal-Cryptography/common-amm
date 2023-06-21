function parseUnits(amount: bigint | number, decimals = 18): bigint {
  return BigInt(amount) * 10n ** BigInt(decimals);
}

export default parseUnits;
