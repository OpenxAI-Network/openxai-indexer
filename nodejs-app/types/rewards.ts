import { Address, SignTypedDataReturnType } from "viem";

export interface Proof {
  proofId: bigint;
  signature: SignTypedDataReturnType;
  claimer: Address;
  amount: bigint;
  basedOn: string[];
}

export interface RewardsStorage {
  [chainId: number]: {
    nextProofId: bigint;
    proofs: {
      [proofId: string]: Proof;
    };
    alreadyClaimed: {
      [basedOn: string]: boolean;
    };
  };
}
