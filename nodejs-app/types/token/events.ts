import { Address } from "viem";
import { EventBase } from "../event-identifier.js";

export type TokenEvent = Transfer | Approval;

export interface Transfer extends EventBase {
  type: "Transfer";
  from: Address;
  to: Address;
  value: bigint;
}

export interface Approval extends EventBase {
  type: "Approval";
  owner: Address;
  spender: Address;
  value: bigint;
}
