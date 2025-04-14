import { Event } from "../types/event.js";
import { Proof } from "../types/rewards.js";
import { Sign } from "../types/sign.js";

export type FilterEventsReturn = Event[];

export type GetProofReturn = Proof;

export type FilterProofsReturn = ({
  chainId: number;
} & Proof)[];

export type FilterSignsReturn = Sign[];
