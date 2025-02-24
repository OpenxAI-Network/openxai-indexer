import { ContractWatcher } from "../../utils/contract-watcher.js";
import { addEvent, getTimestamp } from "../eventHelpers.js";
import { Approval } from "../../types/token/events.js";
import { Storage } from "../../types/storage.js";
import { OpenxAIContract } from "../../contracts/OpenxAI.js";

export function watchApproval(
  contractWatcher: ContractWatcher,
  storage: Storage
) {
  contractWatcher.startWatching("Approval", {
    abi: OpenxAIContract.abi,
    address: OpenxAIContract.address,
    eventName: "Approval",
    strict: true,
    onLogs: async (logs) => {
      await Promise.all(
        logs.map(async (log) => {
          const { args, blockNumber, transactionHash, address, logIndex } = log;

          const event = {
            type: "Approval",
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
          } as Approval;

          await processApproval(event, storage);
        })
      );
    },
  });
}

export async function processApproval(
  event: Approval,
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
