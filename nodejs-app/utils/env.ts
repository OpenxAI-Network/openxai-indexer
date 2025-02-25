export function datadir(): string {
  return process.env.DATADIR ?? "/var/lib/openxai-indexer";
}
