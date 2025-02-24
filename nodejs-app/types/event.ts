import { ClaimerEvent } from "./claimer/events.js";
import { GenesisEvent } from "./genesis/events.js";
import { TokenEvent } from "./token/events.js";

export type Event = ClaimerEvent | GenesisEvent | TokenEvent;
