// The hidden print window, as every document of the app uses it (plans 14 and 15). The backend opens
// a window on `#/print?...&job=<job>`, waits for `print-ready-<job>` and then prints it, or hears
// `print-error-<job>`. The address is read as plain text, so the job name and the numbers in it are
// checked here, once, for every kind of document.

/** The event the backend waits for, with the page height (`PrintReady`). */
export const printReadyEvent = (job: string): string => `print-ready-${job}`;
/** The event that says the page could not be made. */
export const printErrorEvent = (job: string): string => `print-error-${job}`;

/** What a failed print tells the backend: the error codes. */
export interface PrintFailed {
  codes: string[];
}

/** The job becomes part of an event name, so it is letters, digits, `-` and `_` only. */
const JOB = /^[A-Za-z0-9_-]{1,64}$/;

/** A whole number above zero from a plain-text value, or null. Nothing is guessed. */
export function positiveInt(text: string | null): number | null {
  if (text === null || !/^\d{1,15}$/.test(text)) return null;
  const value = Number(text);
  return Number.isSafeInteger(value) && value > 0 ? value : null;
}

/** The job name of an address, even when the rest of it is wrong, so the failure can be told. */
export function jobOf(searchStr: string): string | null {
  const job = new URLSearchParams(searchStr).get('job');
  return job !== null && JOB.test(job) ? job : null;
}
