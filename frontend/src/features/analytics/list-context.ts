// The words and figures a table cell is written with (the grid's own context): the app language,
// Fluent, and money through `formatPaisa` with the app's number grouping. The chart reads cells the
// same way the grid does, so a figure on a bar is the figure in the table (CLAUDE.md rule 1).

import { useLocalization } from '@fluent/react';
import { useMemo } from 'react';
import { formatPaisa, useGrouping } from '../../shared/format';
import type { ListContext } from '../../shared/grid';
import { useLanguage } from '../../shared/i18n';

export function useListContext(): ListContext {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const grouping = useGrouping();
  return useMemo(
    () => ({
      language,
      text: (id, vars) => l10n.getString(id, vars),
      money: (amount, plain = false) => formatPaisa(amount, { grouping, plain }),
    }),
    [language, l10n, grouping],
  );
}
