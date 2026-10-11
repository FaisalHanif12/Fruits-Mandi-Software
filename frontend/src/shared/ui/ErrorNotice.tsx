// Shows what the backend refused, in the app language, with what to do (NFR-USE-13).
// Never shows a raw code (NFR-USE-14). The reference number matches a log line (NFR-OBS-3).
//
// A form that shows some problems next to their own fields (fieldError) passes those field
// paths as `fieldsShown`. The notice then keeps only what no field shows, and shows nothing at
// all when nothing is left, so one problem is never told twice.

import { Localized } from '@fluent/react';
import type { UiError } from '../../bindings/UiError';
import { FALLBACK_ERROR_MESSAGE_ID, errorsOutsideFields } from '../api';
import { Code } from './Code';
import { ErrorMessage } from './ErrorMessage';

interface ErrorNoticeProps {
  error: UiError;
  /** Field paths the screen shows next to their inputs, for example `lots[0].sales[2].qty`. */
  fieldsShown?: readonly string[];
}

export function ErrorNotice({ error, fieldsShown = [] }: ErrorNoticeProps) {
  const items = errorsOutsideFields(error, fieldsShown);
  // An error with no problem listed is a backend slip. The user still gets plain words.
  const empty = error.errors.length === 0;
  if (items.length === 0 && !empty) return null;
  return (
    <div role="alert" className="flex flex-col gap-1 text-due">
      {empty ? (
        <Localized id={FALLBACK_ERROR_MESSAGE_ID}>
          <p className="type-body" />
        </Localized>
      ) : null}
      {items.map((item, index) => (
        <ErrorMessage key={`${item.code}-${String(index)}`} item={item}>
          <p className="type-body" />
        </ErrorMessage>
      ))}
      {error.ref === '' ? null : (
        <Localized id="err-reference" vars={{ ref: error.ref }} elems={{ code: <Code /> }}>
          <p className="type-body text-ink-muted" />
        </Localized>
      )}
    </div>
  );
}
