// The marks of one edited entry as words, ready for a field or a grid cell (plan 44, PRD FR-7.71).
// The history is read only for an entry that says it was edited, and the names of the records it
// mentions (the old account, the old item) are read once each, so "Before 12" is "Before Akram".
// The words come in the language of the app and follow a language switch.

import { useQueries } from '@tanstack/react-query';
import { useMemo } from 'react';
import { useLocalization } from '@fluent/react';
import type { EditEntity } from '../../bindings/EditEntity';
import { accountsGet } from '../accounts/api';
import { formatGrams, isoToDisplayDate, useGrouping } from '../format';
import { displayName, useLanguage } from '../i18n';
import { findMaster } from '../search';
import { stampTimeText } from '../ui/EditStampLine';
import { useEditHistory } from './hooks';
import {
  NO_MARKS,
  markKindOf,
  markMapOf,
  nameKeyOf,
  nameRequestsOf,
  type EntryMark,
} from './marks';
import { auditValueText } from './value-text';

/** One mark in words: the short chip, and the whole sentence for the tip. */
export interface MarkText {
  /** What the field held before, for the chip: "600", "Akram". */
  before: string;
  /** What it holds now. */
  after: string;
  /** "Before: 600 → now: 610, 31-10-2026 4:35 pm, Owner". */
  tip: string;
}

export interface EntryMarks {
  /** The mark of the field at a stored path, or undefined when it was never changed. */
  of: (path: string) => MarkText | undefined;
  /** True when the entry carries at least one mark. */
  any: boolean;
}

const NONE: EntryMarks = { of: () => undefined, any: false };

/**
 * `edited` says whether the entry was ever edited (its row carries the stamp): an entry that was not
 * edited asks nothing. `enabled` lets a screen keep the marks off, for example while the entry is
 * new.
 */
export function useEntryMarks(
  entity: EditEntity,
  id: number | null,
  edited: boolean,
  enabled = true,
): EntryMarks {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const grouping = useGrouping();
  const history = useEditHistory(entity, id ?? 0, enabled && id !== null && edited);
  const marks = useMemo(() => markMapOf(history.data), [history.data]);
  const requests = useMemo(() => nameRequestsOf(marks), [marks]);
  const names = useQueries({
    queries: requests.map((request) => ({
      queryKey: ['edit', 'mark-name', request.key],
      queryFn: async (): Promise<{ name_ur: string; name_en: string; tag?: string } | null> => {
        if (request.kind.kind === 'account') return accountsGet(request.id);
        if (request.kind.kind === 'master') return findMaster(request.kind.scope, request.id);
        return null;
      },
      staleTime: Infinity,
    })),
  });
  const nameOf = useMemo(() => {
    const byKey = new Map<string, string>();
    requests.forEach((request, index) => {
      const record = names[index]?.data;
      if (record !== undefined && record !== null)
        byKey.set(request.key, displayName(record, language));
    });
    return byKey;
  }, [names, requests, language]);

  return useMemo(() => {
    if (marks === NO_MARKS || marks.size === 0) return NONE;
    const ctx = { l10n, language, grouping };
    const words = (path: string, mark: EntryMark, side: 'before' | 'after'): string => {
      const value = mark[side];
      const key = nameKeyOf(path, value);
      if (key !== null) return nameOf.get(key) ?? l10n.getString('edit-mark-other');
      const kind = markKindOf(path);
      if (kind.kind === 'other') return l10n.getString('edit-mark-other');
      if (kind.kind === 'grams' && value.kind === 'number') return formatGrams(value.value);
      return auditValueText(value, ctx);
    };
    const texts = new Map<string, MarkText>();
    for (const [path, mark] of marks) {
      const before = words(path, mark, 'before');
      const after = words(path, mark, 'after');
      const when = l10n.getString('edit-history-version', {
        date: isoToDisplayDate(mark.version.local_date),
        time: stampTimeText(mark.version.local_time, l10n),
        by: mark.version.by,
      });
      const change = l10n.getString('edit-field-mark', { before, after });
      texts.set(path, {
        before,
        after,
        tip: l10n.getString('edit-field-mark-full', { mark: change, when }),
      });
    }
    return { of: (path: string) => texts.get(path), any: true };
  }, [marks, nameOf, l10n, language, grouping]);
}
