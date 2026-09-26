import { activeCatalog, translate } from "./catalog";

const BYTES_PER_UNIT = 1024;
const BYTE_UNITS = ["common.bytes", "common.kilobytes", "common.megabytes", "common.gigabytes"] as const;

export function formatBytes(bytes: number): string {
  let value = bytes;
  let unitIndex = 0;

  while (value >= BYTES_PER_UNIT && unitIndex < BYTE_UNITS.length - 1) {
    value /= BYTES_PER_UNIT;
    unitIndex += 1;
  }

  const fractionDigits = unitIndex === 0 ? 0 : 2;
  const formattedValue = new Intl.NumberFormat(activeCatalog.locale, {
    maximumFractionDigits: fractionDigits,
    minimumFractionDigits: 0,
  }).format(value);
  return `${formattedValue} ${translate(BYTE_UNITS[unitIndex]!)}`;
}
