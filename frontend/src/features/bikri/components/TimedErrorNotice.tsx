// What the backend refused on a save, as the notice of `ErrorNotice`, for 5 seconds (plan 97 item 4, PRD FR-12.52, Umar
// 24-11-2026). It shows when the refusal comes, and again, for another 5 seconds, each time the user tries to save again
// (the `pulse` of the screen, from `NoticePulseContext`). The red marks on the cells and fields that were refused stay
// until they are fixed, and the save is refused again while they stand.

import type { UiError } from '../../../bindings/UiError';
import { useNoticePulse, useTimedNotices } from '../../../shared/hooks';
import { ErrorNotice } from '../../../shared/ui';
import { NOTICE_MS } from '../notices';

interface TimedErrorNoticeProps {
  error: UiError;
  /** Field paths the screen shows next to their inputs (the notice leaves those out, as `ErrorNotice` does). */
  fieldsShown?: readonly string[];
}

/** The key of a refusal: the same refusal answered again is the same note. */
function refusalKey(error: UiError): string {
  return `${error.ref}|${error.errors.map((item) => `${item.code}@${item.field ?? ''}`).join(',')}`;
}

export function TimedErrorNotice({ error, fieldsShown }: TimedErrorNoticeProps) {
  const pulse = useNoticePulse();
  const key = refusalKey(error);
  const shown = useTimedNotices([key], pulse, NOTICE_MS).has(key);
  if (!shown) return null;
  return <ErrorNotice error={error} {...(fieldsShown === undefined ? {} : { fieldsShown })} />;
}
