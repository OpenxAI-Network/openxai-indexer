import { Hex } from "viem";
import { EventsStorage } from "../types/storage.js";
import { publicClients } from "../utils/chain-cache.js";
import { Event } from "../types/event.js";

export async function getTimestamp(chainId: number, blockNumber: bigint): Promise<bigint> {
  const publicClient = publicClients[chainId];
  const block = await publicClient.getBlock({ blockNumber: blockNumber });
  return block.timestamp;
}

export function createChainIfNotExists(events: EventsStorage, chainId: number): void {
  if (!events[chainId]) {
    events[chainId] = {};
  }
}

export function createTransactionIfNotExists(events: EventsStorage, chainId: number, transactionHash: Hex): void {
  createChainIfNotExists(events, chainId);
  if (!events[chainId][transactionHash]) {
    events[chainId][transactionHash] = {};
  }
}

export function addEvent(events: EventsStorage, event: Event): void {
  createTransactionIfNotExists(events, event.chainId, event.transactionHash);
  events[event.chainId][event.transactionHash][event.logIndex] = event;
}
