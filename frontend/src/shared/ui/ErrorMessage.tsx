// One backend error in the app language (Code Standards 4.3, NFR-USE-13). Never shows the raw
// code (NFR-USE-14). Limits such as `max` are grouped like every other number on screen.
// A code with no message in this build gets the plain "something went wrong" words instead.

import { Localized, useLocalization } from '@fluent/react';
import { useCallback, type ReactElement } from 'react';
import type { UiErrorItem } from '../../bindings/UiErrorItem';
import { FALLBACK_ERROR_MESSAGE_ID, errorMessageId, messageVars } from '../api';
import { useGrouping } from '../format';

interface ErrorMessageProps {
  item: UiErrorItem;
  /** The element that holds the words, for example <p className="..." />. */
  children: ReactElement;
}

export function ErrorMessage({ item, children }: ErrorMessageProps) {
  const grouping = useGrouping();
  const { l10n } = useLocalization();
  const id = errorMessageId(item.code);
  if (l10n.getBundle(id) === null) {
    return <Localized id={FALLBACK_ERROR_MESSAGE_ID}>{children}</Localized>;
  }
  return (
    <Localized id={id} vars={messageVars(item.values, grouping)}>
      {children}
    </Localized>
  );
}

/**
 * One backend error as a string in the app language, for a place that needs text and not an
 * element: the tooltip of a grid cell. Same words as ErrorMessage, and the same plain fallback
 * for a code this build has no words for (Code Standards 4.3, NFR-USE-14).
 */
export function useErrorText(): (item: UiErrorItem) => string {
  const grouping = useGrouping();
  const { l10n } = useLocalization();
  // The same function until the language or the grouping changes, so a screen can depend on it.
  return useCallback(
    (item) => {
      const id = errorMessageId(item.code);
      return l10n.getBundle(id) === null
        ? l10n.getString(FALLBACK_ERROR_MESSAGE_ID)
        : l10n.getString(id, messageVars(item.values, grouping));
    },
    [l10n, grouping],
  );
}
