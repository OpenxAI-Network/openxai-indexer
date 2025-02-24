import { Address } from "viem";
import { EventBase } from "../event-identifier.js";

export type ClaimerEvent = TokensClaimed;

export interface TokensClaimed extends EventBase {
  type: "TokensClaimed";
  proofId: bigint;
  account: Address;
  amount: bigint;
}
