// Shared numeric formatting for instrument widgets.
// Large magnitudes (clock in MHz, memory in MB) read as integers; smaller
// instrument values (percent, temperature) keep one decimal. An explicit
// `decimals` override always wins.

export function formatValue(value: number, decimals?: number): string {
  const places = typeof decimals === 'number' ? decimals : Math.abs(value) >= 1000 ? 0 : 1;
  return value.toLocaleString(undefined, {
    minimumFractionDigits: places,
    maximumFractionDigits: places,
  });
}
