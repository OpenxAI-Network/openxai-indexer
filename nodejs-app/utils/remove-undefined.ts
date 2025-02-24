export function removeUndefined<T>(array: (T | undefined)[]): T[] {
  return array.filter((t: T | undefined): t is T => t !== undefined);
}
