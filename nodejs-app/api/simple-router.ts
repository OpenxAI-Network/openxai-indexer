import { Express, json } from "express";
import cors from "cors";
import { Storage } from "../types/storage.js";
import {
  FilterEventsReturn,
  FilterProofsReturn,
  FilterSignsReturn,
  GetProofReturn,
} from "./return-types.js";
import { replacer, reviver } from "../utils/json.js";
import { ObjectFilter, passesObjectFilter } from "./filter.js";
import { historySync } from "../utils/history-sync.js";
import { Address, verifyMessage } from "viem";
import { multichainWatcher } from "../index.js";
import { OpenxAIClaimerContract } from "../contracts/OpenxAIClaimer.js";
import { OpenxAIGenesisContract } from "../contracts/OpenxAIGenesis.js";
import { OpenxAIContract } from "../contracts/OpenxAI.js";
import { sign } from "../utils/rewards-signer.js";
import { Sign } from "../types/sign.js";

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
      }: {
        chainId: number;
        fromBlock: bigint;
        toBlock: bigint;
      } = JSON.parse(JSON.stringify(req.body), reviver);
      historySync(multichainWatcher, chainId, fromBlock, toBlock, [
        OpenxAIClaimerContract.address,
        OpenxAIGenesisContract.address,
        OpenxAIContract.address,
      ]).catch((err) => {
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
      const filterEvents = Object.values(events)
        .map((chainEvents) => Object.values(chainEvents))
        .flat(1)
        .map((transactionEvents) => Object.values(transactionEvents))
        .flat(1)
        .filter((event) => {
          return passesObjectFilter(event, filter);
        });

      res.end(JSON.stringify(filterEvents as FilterEventsReturn, replacer));
    } catch (error: any) {
      res.statusCode = 500;
      res.end(JSON.stringify({ error: error?.message ?? "Unknown error" }));
    }
  });

  app.post(basePath + "getProof", async function (req, res) {
    try {
      const params: { chainId: number; claimer: Address; basedOn: string[] } =
        JSON.parse(JSON.stringify(req.body), reviver);

      const proof = await sign({ ...params, storage: storage });

      res.end(JSON.stringify(proof as GetProofReturn, replacer));
    } catch (error: any) {
      res.statusCode = 500;
      res.end(JSON.stringify({ error: error?.message ?? "Unknown error" }));
    }
  });

  app.post(basePath + "filterProofs", async function (req, res) {
    try {
      const filter: ObjectFilter = JSON.parse(
        JSON.stringify(req.body),
        reviver
      );

      const rewards = await storage.rewards.get();
      const filterProofs = Object.keys(rewards)
        .map(Number)
        .map((chainId) =>
          Object.values(rewards[chainId].proofs).map((proof) => {
            return { chainId, ...proof };
          })
        )
        .flat(1)
        .filter((proof) => {
          return passesObjectFilter(proof, filter);
        });

      res.end(JSON.stringify(filterProofs as FilterProofsReturn, replacer));
    } catch (error: any) {
      res.statusCode = 500;
      res.end(JSON.stringify({ error: error?.message ?? "Unknown error" }));
    }
  });

  app.post(basePath + "uploadSign", async function (req, res) {
    try {
      const params: Omit<Sign, "date"> = JSON.parse(
        JSON.stringify(req.body),
        reviver
      );

      if (
        !verifyMessage({
          address: params.address,
          message: params.message,
          signature: params.signature,
        })
      ) {
        res.statusCode = 400;
        res.end(JSON.stringify({ error: "Signature verification failed." }));
      }

      await storage.signs.update((signs) =>
        signs.push({ ...params, date: Math.round(Date.now() / 1000) })
      );

      res.end();
    } catch (error: any) {
      res.statusCode = 500;
      res.end(JSON.stringify({ error: error?.message ?? "Unknown error" }));
    }
  });

  app.post(basePath + "filterSigns", async function (req, res) {
    try {
      const filter: ObjectFilter = JSON.parse(
        JSON.stringify(req.body),
        reviver
      );

      const signs = await storage.signs.get();
      const filterSigns = Object.values(signs).filter((sign) => {
        return passesObjectFilter(sign, filter);
      });

      res.end(JSON.stringify(filterSigns as FilterSignsReturn, replacer));
    } catch (error: any) {
      res.statusCode = 500;
      res.end(JSON.stringify({ error: error?.message ?? "Unknown error" }));
    }
  });
}
