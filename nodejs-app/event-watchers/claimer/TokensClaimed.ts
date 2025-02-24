import { ContractWatcher } from "../../utils/contract-watcher.js";
import { addEvent, getTimestamp } from "../eventHelpers.js";
import { TokensClaimed } from "../../types/claimer/events.js";
import { Storage } from "../../types/storage.js";
import { OpenxAIClaimerContract } from "../../contracts/OpenxAIClaimer.js";

export function watchTokensClaimed(
  contractWatcher: ContractWatcher,
  storage: Storage
) {
  contractWatcher.startWatching("TokensClaimed", {
    abi: OpenxAIClaimerContract.abi,
    address: OpenxAIClaimerContract.address,
    eventName: "TokensClaimed",
    strict: true,
    onLogs: async (logs) => {
      await Promise.all(
        logs.map(async (log) => {
          const { args, blockNumber, transactionHash, address, logIndex } = log;

          const event = {
            type: "TokensClaimed",
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
          } as TokensClaimed;

          await processTokensClaimed(event, storage);
        })
      );
    },
  });
}

export async function processTokensClaimed(
  event: TokensClaimed,
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
