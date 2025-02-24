export function parseBigInt(from: string): bigint | undefined {
  try {
    return BigInt(from);
  } catch {
    return undefined;
  }
}
