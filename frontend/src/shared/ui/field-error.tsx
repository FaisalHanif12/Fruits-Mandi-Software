// The problems the backend found on one field, ready for <Field error={...}> (Code Standards 4.3).
// <Field error={fieldError(error, 'lots[0].sales[2].qty')}> turns that field red and shows the
// words under it. It returns nothing when the field has no problem, so the field stays normal.
// Several problems on one field show one under the other. No reference number is shown here:
// it belongs to whole-form problems (ErrorNotice).

import type { ReactNode } from 'react';
import type { UiError } from '../../bindings/UiError';
import { errorsForField } from '../api';
import { ErrorMessage } from './ErrorMessage';

export function fieldError(error: UiError | null | undefined, field: string): ReactNode {
  const items = errorsForField(error, field);
  if (items.length === 0) return undefined;
  return (
    <>
      {items.map((item, index) => (
        <ErrorMessage key={`${item.code}-${String(index)}`} item={item}>
          <span className="block" />
        </ErrorMessage>
      ))}
    </>
  );
}
