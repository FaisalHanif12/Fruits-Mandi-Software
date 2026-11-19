// Level 1 of the party ledgers screen (PRD FR-12.43): the buyers or the sellers, one row for each
// party with his code, his name and his balance under the side it is on, by code. The footer is the
// plain total of the debit and of the credit over every party of the filter, the backend's figures
// (PRD FR-19.11). Enter (or a double click) on a row opens his statement; Esc leaves the screen.
//
// Four states: loading, an error with a retry, nothing yet (with a button to the Index) or nothing
// found by the search (with a way out), and the rows. The search text lives here, above the grid, so
// a language switch (which builds the grid again) loses nothing (Code Standards F39).

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { SearchX, UsersRound } from 'lucide-react';
import { useCallback, useMemo, useState } from 'react';
import type { LedgerSide } from '../../../bindings/LedgerSide';
import type { PartyLedgerRow } from '../../../bindings/PartyLedgerRow';
import { uiErrorOf } from '../../../shared/api';
import { PagedList, type ListContext } from '../../../shared/grid';
import { useFocusFirstField } from '../../../shared/hooks';
import { formNavigationProps, useEscape } from '../../../shared/keyboard';
import {
  Button,
  EmptyState,
  Field,
  FilterBar,
  FilterCard,
  ListPage,
  SegmentedControl,
} from '../../../shared/ui';
import { LIST_COLUMNS, listTotals } from '../columns';
import { usePartyLedgersList } from '../hooks';

interface PartyListProps {
  side: LedgerSide;
  onSide: (side: LedgerSide) => void;
  /** Opens the statement of the party. */
  onOpen: (row: PartyLedgerRow) => void;
}

const SIDES: readonly LedgerSide[] = ['buyer', 'seller'];

export function PartyList({ side, onSide, onOpen }: PartyListProps) {
  const navigate = useNavigate();
  const { l10n } = useLocalization();
  const [search, setSearch] = useState('');
  const list = usePartyLedgersList(side, search);
  const containerRef = useFocusFirstField<HTMLDivElement>();
  const [focusId, setFocusId] = useState<string | null>(null);
  useEscape('leave', () => {
    void navigate({ to: '/' });
  });

  const first = list.data?.pages[0];
  // A refusal shows as the state, not as the rows of the filter before it.
  const rows = list.isError ? undefined : list.data?.pages.flatMap((page) => page.rows);
  const searching = search.trim() !== '';

  const totals = useCallback((context: ListContext) => listTotals(first, context), [first]);
  const options = useMemo(
    () =>
      SIDES.map((value) => ({
        value,
        label: <Localized id={`party-ledgers-side-${value}`} />,
      })),
    [],
  );

  const empty = searching ? (
    <EmptyState
      icon={SearchX}
      title={<Localized id="party-ledgers-no-match" />}
      help={<Localized id="party-ledgers-no-match-help" />}
      action={
        <Button
          onClick={() => {
            setSearch('');
          }}
        >
          <Localized id="party-ledgers-clear" />
        </Button>
      }
    />
  ) : (
    <EmptyState
      icon={UsersRound}
      title={<Localized id={`party-ledgers-empty-${side}`} />}
      help={<Localized id="party-ledgers-empty-help" />}
      action={
        <Button
          variant="primary"
          onClick={() => {
            void navigate({ to: '/index/accounts' });
          }}
        >
          <Localized id="party-ledgers-empty-action" />
        </Button>
      }
    />
  );

  return (
    <ListPage
      titleId="party-ledgers-title"
      icon={UsersRound}
      help={
        <Localized id="party-ledgers-help">
          <p />
        </Localized>
      }
      filters={
        <FilterCard ref={containerRef} {...formNavigationProps}>
          <FilterBar
            screen="party_ledgers.list"
            search={{
              value: search,
              onChange: setSearch,
              placeholderId: 'party-ledgers-search',
            }}
          >
            <Field label={<Localized id="party-ledgers-side-label" />} hideLabel>
              {() => (
                <div>
                  <SegmentedControl<LedgerSide>
                    options={options}
                    value={side}
                    groupLabel={l10n.getString('party-ledgers-side-label')}
                    onChange={onSide}
                  />
                </div>
              )}
            </Field>
          </FilterBar>
        </FilterCard>
      }
    >
      <PagedList<PartyLedgerRow>
        columns={LIST_COLUMNS}
        rows={rows}
        getRowId={(row) => String(row.account_id)}
        labelId="party-ledgers-list-label"
        failedId="party-ledgers-list-failed"
        error={list.isError ? uiErrorOf(list.error) : null}
        onRetry={() => {
          void list.refetch();
        }}
        pager={list.pager}
        empty={empty}
        onOpen={onOpen}
        onFocusRow={setFocusId}
        focusRowId={focusId}
        isInactive={(row) => !row.is_active}
        totals={totals}
      />
    </ListPage>
  );
}
