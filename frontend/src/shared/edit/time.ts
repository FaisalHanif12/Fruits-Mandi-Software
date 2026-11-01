// The clock of an edit stamp as words (plan 44, PRD FR-7.71): "4:35 شام" in Urdu, "4:35 pm" in
// English. The backend sends the local time as `HH:MM`; this only chooses the words around it.
// Display only: nothing here is a date calculation.

/** The part of the day for the Urdu words, and am or pm for the English ones. */
export interface ClockParts {
  hour: string;
  minute: string;
  part: 'night' | 'morning' | 'noon' | 'evening';
  meridian: 'am' | 'pm';
}

const MATCH = /^(\d{1,2}):(\d{2})/;

/** The 12-hour clock of `HH:MM`, or null when the text is not a time. */
export function clockPartsOf(localTime: string): ClockParts | null {
  const found = MATCH.exec(localTime);
  if (found === null) return null;
  const hours = Number(found[1]);
  if (!Number.isInteger(hours) || hours < 0 || hours > 23) return null;
  const twelve = hours % 12 === 0 ? 12 : hours % 12;
  const part =
    hours < 4 || hours >= 20 ? 'night' : hours < 12 ? 'morning' : hours < 16 ? 'noon' : 'evening';
  return {
    hour: String(twelve),
    minute: found[2] ?? '00',
    part,
    meridian: hours < 12 ? 'am' : 'pm',
  };
}
