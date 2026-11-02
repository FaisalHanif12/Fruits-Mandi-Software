// How long a question took, for the question log in Settings (plan 51 B2, plan 52). The backend keeps
// whole milliseconds. A time under a second shows in milliseconds; a longer one in seconds with one
// decimal, so "8.4 s (model 8.3 s)" reads at a glance. This is a duration, not money: the rounding is
// for the eye only, and it uses whole numbers like the rest of the app.

/** The unit of a shown duration. The words for the units are in Fluent. */
export type DurationUnit = 'ms' | 's';

export interface ShownDuration {
  value: string;
  unit: DurationUnit;
}

const MILLIS_PER_SECOND = 1000;
const MILLIS_PER_TENTH = 100;
const HALF_A_TENTH = 50;
const TENTHS_PER_SECOND = 10;

/** 12 gives "12 ms"; 8,370 gives "8.4 s". Whole-number maths only, rounded to the nearest tenth. */
export function shownDuration(millis: number): ShownDuration {
  if (millis < MILLIS_PER_SECOND) return { value: String(millis), unit: 'ms' };
  const tenths = Math.trunc((millis + HALF_A_TENTH) / MILLIS_PER_TENTH);
  const seconds = Math.trunc(tenths / TENTHS_PER_SECOND);
  return { value: `${String(seconds)}.${String(tenths % TENTHS_PER_SECOND)}`, unit: 's' };
}
