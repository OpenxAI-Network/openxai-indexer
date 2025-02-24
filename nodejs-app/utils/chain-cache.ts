import { Chain, PublicClient } from "viem";

export const chains = {} as { [chainId: number]: Chain };

export const publicClients = {} as { [chainId: number]: PublicClient };
