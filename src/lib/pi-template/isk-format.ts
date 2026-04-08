const iskFormat = new Intl.NumberFormat("en-US", { maximumFractionDigits: 0 });

export function formatIsk(n: number): string {
  return `${iskFormat.format(n)} ISK`;
}
