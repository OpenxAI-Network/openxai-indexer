import { Hex } from "viem";
import { PersistentJson } from "../utils/persistent-json.js";
import { Event } from "./event.js";
import { RewardsStorage } from "./rewards.js";

export type EventsStorage = {
  [chainId: number]: {
    [transactionHash: Hex]: {
      [logIndex: number]: Event;
    };
  };
};

export interface Storage {
  events: PersistentJson<EventsStorage>;
  rewards: PersistentJson<RewardsStorage>;
}
