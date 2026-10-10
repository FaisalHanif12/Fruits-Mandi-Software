// Shows what the backend refused, in the app language, with what to do (NFR-USE-13).
// Never shows a raw code (NFR-USE-14). The reference number matches a log line (NFR-OBS-3).

import { Localized } from '@fluent/react';
import type { UiError } from '../../bindings/UiError';
import { errorMessageId } from '../api';
import { Code } from './Code';

interface ErrorNoticeProps {
  error: UiError;
}

export function ErrorNotice({ error }: ErrorNoticeProps) {
  return (
    <div role="alert" className="flex flex-col gap-1 text-due">
      {error.errors.map((item, index) => (
        <Localized
          key={`${item.code}-${String(index)}`}
          id={errorMessageId(item.code)}
          vars={item.values}
        >
          <p className="type-body" />
        </Localized>
      ))}
      {error.ref === '' ? null : (
        <Localized id="err-reference" vars={{ ref: error.ref }} elems={{ code: <Code /> }}>
          <p className="type-body text-ink-muted" />
        </Localized>
      )}
    </div>
  );
}
