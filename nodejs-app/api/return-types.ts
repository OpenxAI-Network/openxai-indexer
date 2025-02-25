import { Event } from "../types/event.js";
import { Proof } from "../types/rewards.js";

export type FilterEventsReturn = Event[];

export type GetProofReturn = Proof;

export type FilterProofsReturn = ({
  chainId: number;
} & Proof)[];
