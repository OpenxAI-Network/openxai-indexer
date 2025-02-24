import {
  Abi,
  Chain,
  ContractEventName,
  createPublicClient,
  decodeEventLog,
  Log,
  PublicClient,
  WatchContractEventParameters,
  webSocket,
} from "viem";

import { chains, publicClients } from "./chain-cache.js";
import { normalizeAddress } from "./normalize-address.js";
import { removeUndefined } from "./remove-undefined.js";

export class ContractWatcher {
  public chain: Chain;
  private client: PublicClient;
  private watching: {
    [watchId: string]: {
      start: () => void;
      stop: () => void;
      tryProcess: (logs: Log[]) => void;
    };
  };
  private timeoutError: Set<string>;

  public getWatched(): string[] {
    return Object.keys(this.watching);
  }

  private refresh(): void {
    this.getWatched().forEach((watchId) => {
      this.watching[watchId].stop();
      this.watching[watchId].start();
    });
  }

  constructor({ chain, rpc }: { chain: Chain; rpc: string }) {
    this.chain = chain;
    this.client = createPublicClient({
      chain: this.chain,
      transport: webSocket(`wss://${rpc}`),
    });
    this.watching = {};
    setInterval(() => {
      // Wrapped in a function to have the correct "this"
      try {
        this.refresh();
      } catch (err) {
        console.error(
          `Refreshing contract watcher ${this.chain.id} error: ${err}`
        );
      }
    }, 60 * 60 * 1000); // Start refresh loop to keep connection alive

    // Expose this info for other classes to use
    chains[this.chain.id] = this.chain;
    publicClients[this.chain.id] = this.client;

    this.timeoutError = new Set();
  }

  public startWatching<
    abi extends Abi | readonly unknown[] = Abi,
    eventName extends
      | ContractEventName<abi>
      | undefined = ContractEventName<abi>,
    strict extends boolean | undefined = undefined
  >(
    watchId: string,
    parameters: Omit<
      WatchContractEventParameters<abi, eventName, strict>,
      "onError"
    >
  ): void {
    if (this.watching[watchId]) {
      this.watching[watchId].stop();
    }

    this.watching[watchId] = {
      start: () => {
        const onError = async (err: Error) => {
          if (this.timeoutError.has(watchId)) {
            // Prevents error triggering multiple times
            return;
          }
          this.timeoutError.add(watchId);
          console.error(
            `Watching ${watchId} on chain ${this.chain.id} error: ${err.message}`
          );
          await new Promise((resolve) => setTimeout(resolve, 60 * 1000)); // Wait 1 minute to prevent hitting rate limits on errors
          this.watching[watchId].stop();
          this.timeoutError.delete(watchId);
          this.watching[watchId].start();
        };
        try {
          this.watching[watchId].stop = this.client.watchContractEvent({
            ...parameters,
            onError: onError,
          });
        } catch (err) {
          onError(
            err instanceof Error
              ? err
              : new Error(`Unknown start watching error`)
          );
        }
      },
      stop: () => {},
      tryProcess: (logs) => {
        const decodedLogs = removeUndefined(
          logs
            .filter(
              (log) =>
                parameters.address === undefined ||
                (typeof parameters.address === "string" &&
                  normalizeAddress(log.address) ===
                    normalizeAddress(parameters.address)) ||
                (parameters.address instanceof Array &&
                  parameters.address.some(
                    (address) =>
                      normalizeAddress(log.address) ===
                      normalizeAddress(address)
                  ))
            )
            .map((log) => {
              try {
                return {
                  ...log,
                  ...decodeEventLog({
                    abi: parameters.abi,
                    eventName: parameters.eventName,
                    strict: parameters.strict,
                    topics: log.topics,
                    data: log.data,
                  }),
                };
              } catch {}
            })
        ).filter(
          (log) =>
            !parameters.eventName || log.eventName === parameters.eventName
        ); // For some reason wagmi decode event log also decodes different events

        parameters.onLogs(decodedLogs as any);
      },
    };

    this.watching[watchId].start();
  }

  public processLogs(logs: Log[]): void {
    Object.values(this.watching).forEach((watcher) => watcher.tryProcess(logs));
  }

  public stopWatching(watchId: string): void {
    if (!this.watching[watchId]) {
      throw new Error(`Tried to remove ${watchId}, but its not being watched.`);
    }

    this.watching[watchId].stop();
    delete this.watching[watchId];
  }

  public stopAll(): void {
    this.getWatched().forEach((watchId) => {
      this.stopWatching(watchId);
    });
  }
}
