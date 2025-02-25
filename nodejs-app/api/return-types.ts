import { Event } from "../types/event.js";
import { Proof } from "../types/rewards.js";

export type FilterEventsReturn = Event[];

export type GetProofReturn = `0x${string}`;

export type FilterProofsReturn = ({
  chainId: number;
  proofId: bigint;
} & Proof)[];
