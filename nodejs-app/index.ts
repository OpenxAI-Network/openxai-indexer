import express from "express";
import storageManager from "node-persist";
import { mainnet, sepolia } from "viem/chains";

import { registerRoutes } from "./api/simple-router.js";
import { MultichainWatcher } from "./utils/multichain-watcher.js";
import { PersistentJson } from "./utils/persistent-json.js";
import { EventsStorage, Storage } from "./types/storage.js";
import { watchTokensClaimed } from "./event-watchers/claimer/TokensClaimed.js";
import { watchParticipated } from "./event-watchers/genesis/Participated.js";
import { watchApproval } from "./event-watchers/token/Approval.js";
import { watchTransfer } from "./event-watchers/token/Transfer.js";
import { RewardsStorage } from "./types/rewards.js";
import { datadir } from "./utils/env.js";
import { join } from "path";

export let multichainWatcher: MultichainWatcher;

async function start() {
  // Make contract watcher for each chain (using Infura provider)
  multichainWatcher = new MultichainWatcher([
    {
      chain: mainnet,
      rpc: `mainnet.infura.io/ws/v3/${process.env.INFURA_API_KEY}`,
    },
    {
      chain: sepolia,
      rpc: `sepolia.infura.io/ws/v3/${process.env.INFURA_API_KEY}`,
    },
  ]);

  // Data (memory + json files (synced) currently, could be migrated to a database solution if needed in the future)
  await storageManager.init({
    dir: join(datadir(), "storage"),
  });
  const storage: Storage = {
    events: new PersistentJson<EventsStorage>("events", {}),
    rewards: new PersistentJson<RewardsStorage>("rewards", {
      [mainnet.id]: {
        nextProofId: BigInt(1),
        proofs: {},
      },
      [sepolia.id]: {
        nextProofId: BigInt(1),
        proofs: {},
      },
    }),
  };
  await storage.events.update((_) => {});

  multichainWatcher.forEach((contractWatcher) => {
    watchTokensClaimed(contractWatcher, storage);

    watchParticipated(contractWatcher, storage);

    watchApproval(contractWatcher, storage);
    watchTransfer(contractWatcher, storage);
  });

  let isStopping = false;
  process.on("SIGINT", async () => {
    if (isStopping) {
      // Sigint can be fired multiple times
      return;
    }
    isStopping = true;
    console.log("Stopping...");

    multichainWatcher.forEach((contractWatcher) => {
      contractWatcher.stopAll();
    });
    await Promise.all(
      Object.values(storage).map((storageItem) => {
        return storageItem.update(() => {}); // Save all memory values to disk
      })
    );
    process.exit();
  });

  // Webserver
  const app = express();
  registerRoutes(app, storage);

  var server = app.listen(process.env.PORT ?? 3001, () => {
    const addressInfo = server.address() as any;
    var host = addressInfo.address;
    var port = addressInfo.port;
    console.log(`Webserver started on ${host}:${port}`);
  });
}

start().catch(console.error);
