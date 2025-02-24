import { Express, json } from "express";
import cors from "cors";
import { Storage } from "../types/storage.js";
import { FilterEventsReturn } from "./return-types.js";
import { replacer, reviver } from "../utils/json.js";
import { ObjectFilter, passesObjectFilter } from "./filter.js";
import { historySync } from "../utils/history-sync.js";
import { Address } from "viem";
import { multichainWatcher } from "../index.js";

export function registerRoutes(app: Express, storage: Storage) {
  const basePath = process.env.BASEPATH ?? "/";
  app.use(cors());
  app.use(json());

  app.post(basePath + "sync", async function (req, res) {
    // In case some event logs were missed
    try {
      const {
        chainId,
        fromBlock,
        toBlock,
        addresses,
      }: {
        chainId: number;
        fromBlock: bigint;
        toBlock: bigint;
        addresses?: Address[];
      } = JSON.parse(JSON.stringify(req.body), reviver);
      historySync(
        multichainWatcher,
        chainId,
        fromBlock,
        toBlock,
        addresses
      ).catch((err) => {
        console.error(`Error while executing history sync: ${err}`);
        res.statusCode = 500;
      });
    } catch (err) {
      console.error(`Error interpreting command: ${err}`);
      res.statusCode = 500;
    }
    res.end();
  });

  // Get all events that pass a certain filter
  app.post(basePath + "filterEvents", async function (req, res) {
    try {
      const filter: ObjectFilter = JSON.parse(
        JSON.stringify(req.body),
        reviver
      );

      const events = await storage.events.get();
      const filterTasks = Object.values(events)
        .map((chainEvents) => Object.values(chainEvents))
        .flat(1)
        .map((transactionEvents) => Object.values(transactionEvents))
        .flat(1)
        .filter((event) => {
          return passesObjectFilter(event, filter);
        });

      res.end(JSON.stringify(filterTasks as FilterEventsReturn, replacer));
    } catch (error: any) {
      res.statusCode = 500;
      res.end(JSON.stringify({ error: error?.message ?? "Unknown error" }));
    }
  });
}
