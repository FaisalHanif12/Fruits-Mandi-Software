// Display-only helpers of the backup screen. Nothing here calculates money (CLAUDE.md rule 1). A
// size in bytes is a plain count of bytes, shown as a short size with its unit.

const KB = 1024;
const SIZE_UNITS = ['bytes', 'kb', 'mb', 'gb'] as const;
export type SizeUnit = (typeof SIZE_UNITS)[number];

/**
 * A size in bytes as a number and a unit: 1,536 is 1.5 kb. Under 1 kb it is whole bytes; above
 * that one decimal. The unit is a Fluent message (`backup-size-<unit>`), the number is Latin
 * digits in both languages (PRD FR-7.32).
 */
export function sizeParts(bytes: number): { value: string; unit: SizeUnit } {
  const whole = Math.max(0, Math.trunc(bytes));
  let divisor = 1;
  let at = 0;
  while (whole >= divisor * KB && at < SIZE_UNITS.length - 1) {
    divisor *= KB;
    at += 1;
  }
  const unit = SIZE_UNITS[at] ?? 'bytes';
  if (unit === 'bytes') return { value: String(whole), unit };
  // Tenths by whole-number arithmetic: 1,536 bytes is 15 tenths of a KB, shown "1.5".
  const tenths = Math.trunc((whole * 10) / divisor);
  return { value: `${String(Math.trunc(tenths / 10))}.${String(tenths % 10)}`, unit };
}

/** The Fluent message ID of a size unit. */
export function sizeUnitId(unit: SizeUnit): string {
  return `backup-size-${unit}`;
}

/** A place where a copy is kept, as the Fluent message ID of its name. */
export function kindId(kind: 'local' | 'usb' | 'cloud'): string {
  return `backup-kind-${kind}`;
}

/** A size as words from Fluent, for example "1.5 MB". `say` turns a message ID into text. */
export function sizeText(
  bytes: number,
  say: (id: string, vars?: Record<string, string>) => string,
): string {
  const { value, unit } = sizeParts(bytes);
  return say('backup-size-value', { value, unit: say(sizeUnitId(unit)) });
}
