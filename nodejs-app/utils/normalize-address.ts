import { Address } from "viem";

export function normalizeAddress(address: Address): Address {
  return address.toLowerCase() as Address;
}
