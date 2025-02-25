import { Address, SignTypedDataReturnType } from "viem";

export interface Proof {
  proofId: bigint;
  signature: SignTypedDataReturnType;
  claimer: Address;
  basedOn: string[];
}

export interface RewardsStorage {
  [chainId: number]: {
    nextProofId: bigint;
    proofs: {
      [proofId: string]: Proof;
    };
  };
}
