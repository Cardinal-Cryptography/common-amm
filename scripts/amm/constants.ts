import { parseUnits } from './shared';

export const DEADLINE = '111111111111111111';
export const ONE_WAZERO = parseUnits(1, 12).toString();
export const ONE_HUNDRED_WAZERO = parseUnits(100, 12).toString();
export const ONE_THOUSAND_WAZERO = parseUnits(1_000, 12).toString();
export const ONE_STABLECOIN = parseUnits(1, 6).toString();
export const ONE_HUNDRED_STABLECOIN = parseUnits(100, 6).toString();
export const ONE_THOUSAND_STABLECOIN = parseUnits(1_000, 6).toString();
export const TOTAL_SUPPLY = parseUnits(1_000_000, 18).toString();
export const STABLE_TOTAL_SUPPLY = parseUnits(1_000_000, 6).toString();
export const DUMMY_ADDRESS =
  '0x0000000000000000000000000000000000000000000000000000000000000000';
