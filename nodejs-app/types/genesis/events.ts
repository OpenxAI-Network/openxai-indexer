import { Address } from "viem";
import { EventBase } from "../event-identifier.js";

export type GenesisEvent = Participated;

export interface Participated extends EventBase {
  type: "Participated";
  tier: bigint;
  account: Address;
  amount: bigint;
}
