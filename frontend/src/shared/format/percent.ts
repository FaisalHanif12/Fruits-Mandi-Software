// A percent on screen. The backend keeps a percent as whole basis points (1% = 100 bps), so 6% is
// 600 and 2.5% is 250. This shows it as the text a person writes: "6", "2.5", "0.05". It works on
// the digits as text and never divides, so it can never make a float (Code Standards M3).

/** Basis points as percent text: 600 gives "6", 250 gives "2.5", 5 gives "0.05". */
export function formatBps(bps: number): string {
  const digits = String(Math.trunc(bps)).padStart(3, '0');
  const whole = digits.slice(0, -2);
  const fraction = digits.slice(-2).replace(/0+$/, '');
  return fraction === '' ? whole : `${whole}.${fraction}`;
}
