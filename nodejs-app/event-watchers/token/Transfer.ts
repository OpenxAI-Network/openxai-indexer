import { ContractWatcher } from "../../utils/contract-watcher.js";
import { addEvent, getTimestamp } from "../eventHelpers.js";
import { Transfer } from "../../types/token/events.js";
import { Storage } from "../../types/storage.js";
import { OpenxAIContract } from "../../contracts/OpenxAI.js";

export function watchTransfer(
  contractWatcher: ContractWatcher,
  storage: Storage
) {
  contractWatcher.startWatching("Transfer", {
    abi: OpenxAIContract.abi,
    address: OpenxAIContract.address,
    eventName: "Transfer",
    strict: true,
    onLogs: async (logs) => {
      await Promise.all(
        logs.map(async (log) => {
          const { args, blockNumber, transactionHash, address, logIndex } = log;

          const event = {
            type: "Transfer",
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
          } as Transfer;

          await processTransfer(event, storage);
        })
      );
    },
  });
}

export async function processTransfer(
  event: Transfer,
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
