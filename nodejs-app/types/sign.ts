import { Address, SignMessageReturnType } from "viem";

export interface Sign {
  address: Address;
  message: string;
  signature: SignMessageReturnType;
  date: number;
}

export type SignStorage = Sign[];
