// The history of one garden (PRD FR-12.37, FR-7.67, FR-19.11): its installments, running costs,
// sales (in our shop and in other shops), fruit taken from its lots and its closing, in the order
// the backend sends (oldest day first), each with what it did to the running balance and the
// balance after it. The list loads page by page with a button for the next page. The row that has
// focus is shown in full under the list: each partner's part of a payment and where it came from
// (C23), or each partner's part of the closing (C26). A row can be cancelled with a reason, while
// the garden is open and its day is open.
//
// Four states: loading, an error with a retry, nothing yet (a helpful message), and the rows. A
// cancelled entry stays in the list, marked in words.

import { Localized, useLocalization } from '@fluent/react';
import { History } from 'lucide-react';
import { useState } from 'react';
import { useRouter } from '@tanstack/react-router';
import { NO_ENTRY_FLAGS, useFixDayFlow } from '../../../shared/edit';
import { ListGrid, Pager, useListPaging } from '../../../shared/grid';
import type { PagerState } from '../../../shared/hooks';
import { SHORTCUTS } from '../../../shared/keyboard';
import {
  Card,
  EmptyState,
  EntryActions,
  EntryPartsList,
  Notice,
  ShareAmounts,
} from '../../../shared/ui';
import { HISTORY_COLUMNS } from '../list-columns';
import type { GardenHistoryRow } from '../types';
import { CancelGardenEntryDialog } from './CancelGardenEntryDialog';

interface GardenHistorySectionProps {
  history: readonly GardenHistoryRow[];
  /** How many rows the garden has in all. */
  count: number;
  /** Where the history is: the page on screen and the page buttons. */
  pager: PagerState;
  /** Edit was pressed on an entry whose day is open (or was reopened for it): the page edits it. */
  onEdit: (row: GardenHistoryRow) => void;
}

/** A stable ID of a row: the entry's id where it has one, else its place in the list. */
function rowKey(row: GardenHistoryRow, index: number): string {
  return row.entry_id === null ? `i${String(index)}` : `e${String(row.entry_id)}`;
}

/** The rows that are an entry of the garden itself; a sale, fruit taken and the closing are not. */
function isGardenEntry(row: GardenHistoryRow): boolean {
  return (
    (row.kind === 'installment' || row.kind === 'cost' || row.kind === 'other_sale') &&
    row.entry_id !== null &&
    row.version !== null
  );
}

export function GardenHistorySection({ history, count, pager, onEdit }: GardenHistorySectionProps) {
  const { l10n } = useLocalization();
  const router = useRouter();
  const fix = useFixDayFlow();
  const paging = useListPaging(pager);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [cancelRow, setCancelRow] = useState<GardenHistoryRow | null>(null);
  const keys = new Map(history.map((row, index) => [row, rowKey(row, index)]));
  const keyOf = (row: GardenHistoryRow) => keys.get(row) ?? '';
  const selected = history.find((row) => keyOf(row) === selectedId) ?? null;
  const canCancel =
    selected !== null &&
    !selected.cancelled &&
    selected.can_cancel &&
    selected.entry_id !== null &&
    selected.version !== null;

  return (
    <Card compact icon={History} title={<Localized id="gardens-history-title" />}>
      {history.length === 0 ? (
        <EmptyState
          icon={History}
          title={<Localized id="gardens-history-empty" />}
          help={<Localized id="gardens-history-empty-help" vars={{ key: SHORTCUTS.save.label }} />}
        />
      ) : (
        <>
          <div className="flex flex-wrap items-center justify-between gap-2">
            <Localized
              id="gardens-history-count"
              vars={{ shown: String(history.length), total: String(count) }}
            >
              <p className="type-body text-ink-muted" />
            </Localized>
            <EntryActions
              flags={
                selected === null
                  ? NO_ENTRY_FLAGS
                  : {
                      ...selected,
                      can_edit: selected.can_edit && isGardenEntry(selected),
                      can_cancel: canCancel,
                    }
              }
              busy={fix.busy}
              onEdit={() => {
                if (selected === null) return;
                if (!selected.needs_reopen) {
                  onEdit(selected);
                  return;
                }
                // A closed day: the days are reopened first, then the entry is edited (FR-21.4).
                fix.start({
                  date: selected.entry_date,
                  ownerHref: router.state.location.href,
                  then: () => {
                    onEdit(selected);
                  },
                });
              }}
              onCancel={() => {
                setCancelRow(selected);
              }}
            />
          </div>
          <ListGrid<GardenHistoryRow>
            columns={HISTORY_COLUMNS}
            rows={history}
            getRowId={keyOf}
            labelId="gardens-history-label"
            onOpen={(row) => {
              setSelectedId(keyOf(row));
            }}
            onFocusRow={setSelectedId}
            focusRowId={selectedId}
            heightMode="auto"
            isInactive={(row) => row.cancelled}
            paging={paging}
          />
          <Pager pager={pager} />
        </>
      )}
      {selected === null ? null : (
        <div className="flex flex-col gap-2 rounded-2 border border-border p-3">
          {selected.cancelled ? (
            <Localized
              id="gardens-entry-cancelled-reason"
              vars={{ reason: selected.cancel_reason ?? '' }}
            >
              <p className="type-label text-due" role="status" />
            </Localized>
          ) : null}
          {selected.parts.length === 0 ? null : (
            <>
              <Localized id="gardens-parts-title">
                <h3 className="type-label text-ink-muted" />
              </Localized>
              <EntryPartsList
                parts={selected.parts}
                label={l10n.getString('gardens-parts-title')}
              />
              {selected.journal_entry_id === null ? (
                <Notice tone="info">
                  <Localized id="ventures-parts-no-posting" />
                </Notice>
              ) : null}
            </>
          )}
          {selected.credits.length === 0 ? null : (
            <>
              <Localized id="gardens-close-credits-title">
                <h3 className="type-label text-ink-muted" />
              </Localized>
              <ShareAmounts
                credits={selected.credits}
                label={l10n.getString('gardens-close-credits-title')}
              />
            </>
          )}
        </div>
      )}
      {fix.dialog}
      <CancelGardenEntryDialog
        row={cancelRow}
        onClose={() => {
          setCancelRow(null);
        }}
      />
    </Card>
  );
}
