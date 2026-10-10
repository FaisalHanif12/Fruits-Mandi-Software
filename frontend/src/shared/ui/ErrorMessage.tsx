// One backend error in the app language (Code Standards 4.3, NFR-USE-13). Never shows the raw
// code (NFR-USE-14). Limits such as `max` are grouped like every other number on screen.

import { Localized } from '@fluent/react';
import type { ReactElement } from 'react';
import type { UiErrorItem } from '../../bindings/UiErrorItem';
import { errorMessageId, messageVars } from '../api';
import { useGrouping } from '../format';

interface ErrorMessageProps {
  item: UiErrorItem;
  /** The element that holds the words, for example <p className="..." />. */
  children: ReactElement;
}

export function ErrorMessage({ item, children }: ErrorMessageProps) {
  const grouping = useGrouping();
  return (
    <Localized id={errorMessageId(item.code)} vars={messageVars(item.values, grouping)}>
      {children}
    </Localized>
  );
}
