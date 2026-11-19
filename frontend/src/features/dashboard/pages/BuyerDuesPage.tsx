// The buyer dues page (route /dues; PRD FR-9.4, FR-19.9, FR-19.11, plan 76 point 9): the tile گاہکوں کے ذمے opens
// it. One row for each buyer who owes: his code, name with tag, city, what he owes now (with بنام) and
// the day his balance last changed, with the grand total under the columns. There are no aging buckets:
// R20 in the reports keeps them. The rows, the page counts and the total are the backend's, over the
// whole filter (the total is the tile's number and the R20 total); the rows on screen are never added up
// (CLAUDE.md rule 1).
//
// - One search box (code, running number, name in either script) and the buyer filter, which starts at سب
//   (PRD FR-19.1, plan 79 item 1): every buyer and the full total; picking one buyer narrows the page to him.
//   The filters live here, above the table, so a language switch loses nothing.
// - ↑ ↓ choose a buyer; Enter opens his history: the party ledgers statement, one row for each entry with the
//   opening balance first (plan 78 B1, B2, FR-19.16). Page Up and Page Down turn the pages.
// - Four states: loading, an error with a retry, nobody owes anything (or nothing matches the
//   search, with a way out), and the rows.

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { Printer, SearchX, Users } from 'lucide-react';
import { useCallback, useMemo, useState } from 'react';
import type { BuyerDuesRow } from '../../../bindings/BuyerDuesRow';
import { uiErrorOf } from '../../../shared/api';
import { PagedList, type ListContext } from '../../../shared/grid';
import { useFocusFirstField } from '../../../shared/hooks';
import { SHORTCUTS, formNavigationProps, useEscape, useShortcut } from '../../../shared/keyboard';
import {
  AccountPicker,
  Button,
  EmptyState,
  Field,
  FilterBar,
  ListPage,
  type PickerValue,
} from '../../../shared/ui';
import { ExcelExportButton, ExcelExportNotices } from '../../../shared/export';
import { useLanguage } from '../../../shared/i18n';
import { TablePrintPreview, useTableExcel } from '../../reports';
import { dashboardBuyerDuesPrint, dashboardBuyerDuesPrintData } from '../api';
import { BUYER_DUES_COLUMNS } from '../dues-columns';
import { useBuyerDues } from '../dues-hooks';

/**
 * The id of a row. The grid also asks for the id of the pinned totals row, which has no buyer: it gets a
 * name of its own, so the totals row is never taken for a buyer.
 */
function rowId(row: BuyerDuesRow): string {
  const { party } = row as { party?: BuyerDuesRow['party'] };
  return party === undefined ? 'totals' : String(party.account_id);
}

export function BuyerDuesPage() {
  const navigate = useNavigate();
  const [search, setSearch] = useState('');
  const [party, setParty] = useState<PickerValue | null>(null);
  const [focusId, setFocusId] = useState<string | null>(null);
  const [view, setView] = useState<'table' | 'preview'>('table');
  const list = useBuyerDues({ search: search.trim(), partyId: party === null ? null : party.id });
  const containerRef = useFocusFirstField<HTMLDivElement>();
  const language = useLanguage();
  // Excel: every buyer of the search on screen, sorted as the list is.
  const exporter = useTableExcel('buyer-dues', (sheet) => ({
    kind: 'buyer_dues',
    party_id: party === null ? null : party.id,
    search: search.trim() === '' ? null : search.trim(),
    sort: list.sort,
    language: sheet,
    paper: null,
  }));
  useEscape(
    'leave',
    () => {
      setView('table');
    },
    view === 'preview',
  );
  useShortcut(SHORTCUTS.print, () => {
    if (view === 'table') setView('preview');
  });

  const first = list.data?.pages[0];
  // A refusal shows as the state, not as the rows of the search before it.
  const rows = list.isError ? undefined : list.data?.pages.flatMap((page) => page.rows);
  const filtering = search.trim() !== '' || party !== null;

  const openLedger = useCallback(
    (row: BuyerDuesRow) => {
      void navigate({
        to: '/bikri/party-ledgers',
        search: { side: 'buyer', party: row.party.account_id },
      });
    },
    [navigate],
  );

  // The totals row is the backend's, over every buyer of the search and not over the rows loaded.
  const totals = useCallback(
    (context: ListContext) => {
      const total = first?.total_paisa;
      return {
        labelField: 'buyer',
        values:
          total === undefined
            ? {}
            : {
                buyer: context.text('glossary-meezan'),
                balance: context.money(total),
                // What the buyers owe is on the بنام side; nobody owing is no side at all.
                'balance:unit': context.money(total) === '0' ? '' : context.text('report-side-dr'),
              },
      };
    },
    [first],
  );

  const empty = useMemo(
    () =>
      filtering ? (
        <EmptyState
          icon={SearchX}
          title={<Localized id="dues-no-match" />}
          help={<Localized id="dues-no-match-help" />}
          action={
            <Button
              onClick={() => {
                setSearch('');
                setParty(null);
              }}
            >
              <Localized id="dues-clear" />
            </Button>
          }
        />
      ) : (
        <EmptyState
          icon={Users}
          title={<Localized id="dues-empty" />}
          help={<Localized id="dues-empty-help" />}
        />
      ),
    [filtering],
  );

  if (view === 'preview') {
    // The print is for the search and the order on screen, every buyer of them, not one page.
    const printInput = (printLanguage: 'ur' | 'en', paper: 'a4' | 'a5' | 'mm80') => ({
      party_id: party === null ? null : party.id,
      search: search.trim() === '' ? null : search.trim(),
      sort: list.sort,
      language: printLanguage,
      paper,
    });
    return (
      <ListPage icon={Users} titleId="dues-title">
        <TablePrintPreview
          exporter={exporter}
          scope={['buyer_dues', party?.id ?? null, search.trim(), list.sort]}
          read={(printLanguage, paper) =>
            dashboardBuyerDuesPrintData(printInput(printLanguage, paper))
          }
          send={(to, printLanguage, paper) =>
            dashboardBuyerDuesPrint({ dues: printInput(printLanguage, paper), to, copies: null })
          }
          onBack={() => {
            setView('table');
          }}
        />
      </ListPage>
    );
  }

  return (
    <ListPage
      icon={Users}
      titleId="dues-title"
      help={
        <Localized id="dues-help">
          <p />
        </Localized>
      }
      actions={
        <>
          <Button
            shortcut={SHORTCUTS.print}
            onClick={() => {
              setView('preview');
            }}
          >
            <Printer aria-hidden="true" className="size-icon-md" />
            <Localized id="report-preview" />
          </Button>
          <ExcelExportButton exporter={exporter} language={language} />
        </>
      }
      filters={
        <div ref={containerRef} {...formNavigationProps}>
          <FilterBar
            screen="dashboard.buyer_dues"
            search={{ value: search, onChange: setSearch, placeholderId: 'dues-search' }}
          >
            <Field label={<Localized id="dashboard-dues-col-party" />}>
              {(control) => (
                <AccountPicker
                  {...control}
                  canAdd={false}
                  allChoice
                  scope="buyer"
                  showCard={false}
                  value={party}
                  onChange={(hit) => {
                    setParty(
                      hit === null
                        ? null
                        : {
                            id: hit.id,
                            name_ur: hit.name_ur,
                            name_en: hit.name_en,
                            code: hit.code,
                          },
                    );
                  }}
                />
              )}
            </Field>
          </FilterBar>
        </div>
      }
    >
      <ExcelExportNotices exporter={exporter} />
      <PagedList<BuyerDuesRow>
        columns={BUYER_DUES_COLUMNS}
        rows={rows}
        getRowId={rowId}
        labelId="dues-label"
        failedId="dues-failed"
        error={list.isError ? uiErrorOf(list.error) : null}
        onRetry={() => {
          void list.refetch();
        }}
        pager={list.pager}
        empty={empty}
        onOpen={openLedger}
        onFocusRow={setFocusId}
        focusRowId={focusId}
        totals={totals}
      />
    </ListPage>
  );
}
