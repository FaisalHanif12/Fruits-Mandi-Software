// The words and figures a table cell is written with (the grid's own context): the app language,
// Fluent, and money through `formatPaisa` with the app's number grouping. A screen that writes a cell as words
// (the items of a statement row, the analytics chart) reads it the same way the grid does, so a figure in a
// sentence is the figure in the table (CLAUDE.md rule 1).

import { useLocalization } from '@fluent/react';
import { useMemo } from 'react';
import { formatPaisa, formatPaisaValue, useGrouping } from '../format';
import { useLanguage } from '../i18n';
import type { ListContext } from './ListGrid';

export function useListContext(): ListContext {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const grouping = useGrouping();
  return useMemo(
    () => ({
      language,
      text: (id, vars) => l10n.getString(id, vars),
      money: (amount, plain = false) => formatPaisa(amount, { grouping, plain }),
      signedMoney: (value) => formatPaisaValue(value, { grouping, plain: false }),
    }),
    [language, l10n, grouping],
  );
}
