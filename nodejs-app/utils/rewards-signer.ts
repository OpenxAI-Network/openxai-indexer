import {
  Address,
  createWalletClient,
  formatUnits,
  http,
  isHex,
  parseUnits,
} from "viem";
import { OpenxAIClaimerContract } from "../contracts/OpenxAIClaimer.js";
import { mainnet, sepolia } from "viem/chains";
import { privateKeyToAccount } from "viem/accounts";
import { EventsStorage, Storage } from "../types/storage.js";
import { readFile } from "fs/promises";
import { datadir } from "./env.js";
import { join } from "path";
import { replacer, reviver } from "./json.js";
import { EventIdentifier } from "../types/event-identifier.js";
import { Proof } from "../types/rewards.js";
import projectsRaw from "./projects.json" with {type:"json"};

const projects = projectsRaw as {
  fundingGoal: string;
  backersRewards: string;
  flashBonus: string;
  status: String;
}[];

const MILESTONES = projects.map((p) => {
  return {
    rate:
      (parseInt(p.backersRewards) + parseInt(p.flashBonus)) /
      parseInt(p.fundingGoal),
    completed: p.status === "Completed",
  };
});

async function calculateReward({
  chainId,
  claimer,
  basedOn,
  events,
}: {
  chainId: number;
  claimer: Address;
  basedOn: string[];
  events: EventsStorage;
}): Promise<bigint> {
  let reward = BigInt(0);

  for (let i = 0; i < basedOn.length; i++) {
    const base = basedOn[i];
    if (base.startsWith("event:")) {
      const eventId = JSON.parse(
        base.replace("event:", ""),
        reviver
      ) as EventIdentifier;
      if (eventId.chainId !== chainId) {
        throw Error(`Chain id of event ${eventId} does not match ${chainId}`);
      }

      const event =
        events[eventId.chainId][eventId.transactionHash][eventId.logIndex];
      if (!event) {
        throw Error(`Event ${eventId} does not exist`);
      }

      if (event.type === "Participated") {
        if (event.account.toLowerCase() !== claimer.toLowerCase()) {
          throw Error(
            `Participation event ${event} does not match claimer ${claimer}`
          );
        }

        const milestone = MILESTONES.at(Number(event.tier));
        if (!milestone) {
          throw Error(`Milestone ${event.tier.toString()} not found`);
        }
        if (!milestone.completed && chainId !== sepolia.id) {
          // Testnet allows claiming of all milestones
          throw Error(`Milestone ${event.tier.toString()} not completed yet`);
        }
        reward += parseUnits(
          (
            parseFloat(formatUnits(event.amount, 6)) * milestone.rate
          ).toString(),
          18
        );
      } else {
        throw Error(`Event ${eventId} is not rewarded`);
      }
    } else {
      throw Error(`Unknown identifier for base ${base}`);
    }
  }

  return reward;
}

async function getSigner({ chainId }: { chainId: number }) {
  let chain = chainId === mainnet.id ? mainnet : sepolia;
  if (chain.id != chainId) {
    throw Error(`Unknown chain ${chainId}`);
  }

  const privateKey = await readFile(join(datadir(), "signer.key"), {
    encoding: "utf-8",
  });

  if (!isHex(privateKey)) {
    throw Error("Invalid signer private key.");
  }

  return createWalletClient({
    account: privateKeyToAccount(privateKey),
    chain,
    transport: http(),
  });
}

export async function sign({
  chainId,
  claimer,
  basedOn,
  storage,
}: {
  chainId: number;
  claimer: Address;
  basedOn: string[];
  storage: Storage;
}): Promise<Proof> {
  const domain = {
    name: "OpenxAIClaiming",
    version: "1",
    chainId,
    verifyingContract: OpenxAIClaimerContract.address,
  } as const;
  const types = {
    Claim: [
      { name: "proofId", type: "uint256" },
      { name: "claimer", type: "address" },
      { name: "amount", type: "uint256" },
    ],
  } as const;

  const events = await storage.events.get();
  const amount = await calculateReward({ chainId, claimer, basedOn, events });
  const signer = await getSigner({ chainId });

  let alreadyClaimed: string | undefined = undefined;
  await storage.rewards.update(async (rewards) => {
    const chainRewards = rewards[chainId];
    for (let i = 0; i < basedOn.length; i++) {
      const base = basedOn[i];
      if (base.startsWith("event:")) {
        const eventId = JSON.parse(
          base.replace("event:", ""),
          reviver
        ) as EventIdentifier;
        if (eventId.chainId !== chainId) {
          throw Error(`Chain id of event ${eventId} does not match ${chainId}`);
        }

        const event =
          events[eventId.chainId][eventId.transactionHash][eventId.logIndex];
        if (!event) {
          throw Error(`Event ${eventId} does not exist`);
        }

        const normalizedBasedOn = JSON.stringify(
          {
            chainId: event.chainId,
            transactionHash: event.transactionHash,
            logIndex: event.logIndex,
          },
          replacer
        );
        if (chainRewards.alreadyClaimed[normalizedBasedOn]) {
          alreadyClaimed = normalizedBasedOn;
        }

        // If a later error occurs, the based on events will not revert back to false
        chainRewards.alreadyClaimed[normalizedBasedOn] = true;
      }
    }
  });
  if (alreadyClaimed) {
    throw Error(`Event ${alreadyClaimed} already claimed`);
  }

  let proofId = BigInt(0);
  await storage.rewards.update((rewards) => {
    const chainRewards = rewards[chainId];
    proofId = ++chainRewards.nextProofId;
  });

  if (proofId === BigInt(0)) {
    /// Typescript is not able to deduct that it is always set
    throw Error("Unable to get proof id.");
  }

  let signature = await signer.signTypedData({
    domain: domain,
    types: types,
    primaryType: "Claim",
    message: {
      proofId,
      claimer,
      amount,
    },
  });

  const proof: Proof = { proofId, signature, claimer, amount, basedOn };
  await storage.rewards.update(async (rewards) => {
    const chainRewards = rewards[chainId];
    if (chainRewards.proofs[proofId.toString()]) {
      throw Error(`Proof id already used.`);
    }
    chainRewards.proofs[proofId.toString()] = proof;
  });

  return proof;
}
