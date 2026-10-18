// "What you changed" in a posting open for edit (PRD FR-12.18, plan 16): the audit comparison the
// user sees before he saves. Each change is one sentence in words, the old text next to the new
// text; a line added or removed is told as well. Under it, the numbers of the selected lot before
// and after, both the backend's: the stored ones and the preview's (never worked out here).
// Nothing changed yet says so. The same changes are marked in the grids (corner mark and tooltip)
// so colour is never the only signal (NFR-USE-12).

import { Localized, useLocalization } from '@fluent/react';
import type { Paisa } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { Card, Num, Table, type TableColumn } from '../../../shared/ui';
import type { ChangeItem } from '../edit-changes';
import { HEADER_FIELD_LABEL, LOT_FIELD_LABEL, SALE_FIELD_LABEL, fixedWord } from '../edit-fields';

/** The four numbers compared. Both the backend's: the loaded entry and the changed one. */
export interface Figures {
  gross: Paisa;
  deductions: Paisa;
  net: Paisa;
  buyer_total: Paisa;
}

interface NumberRow {
  labelId: string;
  before: Paisa | null;
  after: Paisa | null;
}

function numberRow(labelId: string, was: Paisa | undefined, now: Paisa | undefined): NumberRow {
  return { labelId, before: was ?? null, after: now ?? null };
}

/** The four numbers compared, lot by lot. */
function numberRows(before: Figures | null, after: Figures | null): NumberRow[] {
  return [
    numberRow('glossary-kham-bikri', before?.gross, after?.gross),
    numberRow('bikri-summary-deductions', before?.deductions, after?.deductions),
    numberRow('glossary-safi-bikri', before?.net, after?.net),
    numberRow('bikri-summary-buyer-total', before?.buyer_total, after?.buyer_total),
  ];
}

interface EditChangesCardProps {
  items: readonly ChangeItem[];
  /** The numbers of the selected lot as loaded (before) and as changed (after). */
  before: Figures | null;
  after: Figures | null;
}

export function EditChangesCard({ items, before, after }: EditChangesCardProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();

  const where = (item: ChangeItem): string =>
    item.scope === 'header'
      ? l10n.getString('bikri-edit-where-header')
      : item.scope === 'lot'
        ? l10n.getString('bikri-edit-where-lot', { lot: String(item.lotNo) })
        : l10n.getString('bikri-edit-where-line', {
            lot: String(item.lotNo),
            line: String(item.lineNo ?? 0),
          });

  const labelOf = (item: ChangeItem): string => {
    const id =
      item.field === null
        ? undefined
        : item.scope === 'header'
          ? HEADER_FIELD_LABEL[item.field]
          : item.scope === 'lot'
            ? LOT_FIELD_LABEL[item.field]
            : SALE_FIELD_LABEL[item.field];
    return id === undefined ? (item.field ?? '') : l10n.getString(id);
  };

  const valueOf = (item: ChangeItem, value: string, record: ChangeItem['beforeRecord']): string => {
    if (record !== null) return displayName(record, language);
    if (value.trim() === '') return l10n.getString('bikri-edit-empty');
    const word = item.field === null ? null : fixedWord(item.scope, item.field, value);
    return word === null ? value : l10n.getString(word);
  };

  const numbers = numberRows(before, after);
  const columns: readonly TableColumn<NumberRow>[] = [
    {
      id: 'what',
      header: l10n.getString('bikri-edit-compare-what'),
      cell: (row) => l10n.getString(row.labelId),
    },
    {
      id: 'before',
      header: l10n.getString('bikri-edit-before'),
      kind: 'number',
      cell: (row) => (row.before === null ? '' : <Num paisa={row.before} />),
    },
    {
      id: 'after',
      header: l10n.getString('bikri-edit-after'),
      kind: 'number',
      cell: (row) => (row.after === null ? '' : <Num paisa={row.after} />),
    },
  ];

  return (
    <Card compact title={<Localized id="bikri-edit-changes-title" />}>
      {items.length === 0 ? (
        <Localized id="bikri-edit-no-changes">
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : (
        <ul className="flex flex-col gap-1">
          {items.map((item, index) => (
            <li
              key={`${item.rowId}-${item.field ?? 'row'}-${String(index)}`}
              className="type-body text-ink"
            >
              {item.field === null ? (
                <Localized
                  id="bikri-edit-removed-line"
                  vars={{
                    where: where(item),
                    buyer:
                      item.beforeRecord === null ? '' : displayName(item.beforeRecord, language),
                  }}
                >
                  <span />
                </Localized>
              ) : item.kind === 'added' ? (
                <Localized
                  id="bikri-edit-added"
                  vars={{
                    where: where(item),
                    field: labelOf(item),
                    after: valueOf(item, item.after, item.afterRecord),
                  }}
                >
                  <span />
                </Localized>
              ) : (
                <Localized
                  id="bikri-edit-change"
                  vars={{
                    where: where(item),
                    field: labelOf(item),
                    before: valueOf(item, item.before, item.beforeRecord),
                    after: valueOf(item, item.after, item.afterRecord),
                  }}
                >
                  <span />
                </Localized>
              )}
            </li>
          ))}
        </ul>
      )}
      <Table
        label={l10n.getString('bikri-edit-changes-title')}
        columns={columns}
        rows={numbers}
        rowKey={(row) => row.labelId}
      />
    </Card>
  );
}
