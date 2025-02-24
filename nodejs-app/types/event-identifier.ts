import { Address, Hex } from "viem";

export interface EventIdentifier {
  chainId: number;
  transactionHash: Hex;
  logIndex: number;
}

export interface EventBase extends EventIdentifier {
  blockNumber: bigint;
  address: Address;
  timestamp: bigint;
}
