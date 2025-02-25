import { Address, SignTypedDataReturnType } from "viem";

export interface Proof {
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
