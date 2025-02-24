import { ContractWatcher } from "../../utils/contract-watcher.js";
import { addEvent, getTimestamp } from "../eventHelpers.js";
import { Participated } from "../../types/genesis/events.js";
import { Storage } from "../../types/storage.js";
import { OpenxAIGenesisContract } from "../../contracts/OpenxAIGenesis.js";

export function watchParticipated(
  contractWatcher: ContractWatcher,
  storage: Storage
) {
  contractWatcher.startWatching("Participated", {
    abi: OpenxAIGenesisContract.abi,
    address: OpenxAIGenesisContract.address,
    eventName: "Participated",
    strict: true,
    onLogs: async (logs) => {
      await Promise.all(
        logs.map(async (log) => {
          const { args, blockNumber, transactionHash, address, logIndex } = log;

          const event = {
            type: "Participated",
            blockNumber,
            transactionHash,
            chainId: contractWatcher.chain.id,
            address: address,
            logIndex: logIndex,
            timestamp: await getTimestamp(
              contractWatcher.chain.id,
              blockNumber
            ),
            ...args,
          } as Participated;

          await processParticipated(event, storage);
        })
      );
    },
  });
}

export async function processParticipated(
  event: Participated,
  storage: Storage
): Promise<void> {
  await storage.events.update((events) => {
    if (
      events[event.chainId]?.[event.transactionHash]?.[event.logIndex] !==
      undefined
    ) {
      return;
    }

    addEvent(events, event);
  });
}
