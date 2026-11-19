// The seller dues page (route /dues/sellers; PRD FR-9.6, FR-9.1, FR-19.1, FR-19.11, plan 79 items 1 and 2a): the tiles
// بیوپاریوں کو ادائیگی باقی and بیوپاریوں کے ذمے open it, each on its own side. One row for each seller with
// a balance: his code, name with tag, city, what the shop owes him, what he owes the shop beyond his advances,
// his balance with its side word and the day it last changed. At the top are the totals of both sides and of
// the open advances; each is a tile's figure. The rows, the counts and the totals are the backend's, over the
// whole filter; the screen adds nothing up (CLAUDE.md rule 1).
//
// - The side filter (سب, the shop owes, the seller owes) and the seller filter both start at سب (PRD FR-19.1).
//   The side is in the address, so a language switch and the way back land on the same side; the seller is
//   kept here, above the grid, so a language switch loses nothing.
// - ↑ ↓ choose a seller; Enter opens his whole history: the party ledgers statement, the opening balance
//   first, one row for each entry (FR-19.16). Page Up and Page Down turn the pages.
// - Four states: loading, an error with a retry, nobody has a balance (or nothing matches the filters, with a
//   way back to سب), and the rows.

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate, useSearch } from '@tanstack/react-router';
import { SearchX, UserRound } from 'lucide-react';
import { useCallback, useState } from 'react';
import type { SellerDuesRow } from '../../../bindings/SellerDuesRow';
import type { SellerDuesSide } from '../../../bindings/SellerDuesSide';
import type { Language } from '../../../bindings/Language';
import type { Paper } from '../../../bindings/Paper';
import { ExcelExportNotices } from '../../../shared/export';
import { TablePrintPreview, useTableExcel } from '../../reports';
import { PrintButtons, usePrintView } from '../print-view';
import { uiErrorOf } from '../../../shared/api';
import { PagedList, type ListContext } from '../../../shared/grid';
import { formNavigationProps } from '../../../shared/keyboard';
import {
  AccountPicker,
  Button,
  EmptyState,
  Fact,
  Field,
  FilterCard,
  ListPage,
  Num,
  Select,
  type PickerValue,
} from '../../../shared/ui';
import { SELLER_DUES_COLUMNS } from '../seller-dues-columns';
import { SELLER_DUES_SIDES, parseSellerDuesSearch } from '../seller-dues-search';
import { dashboardSellerDuesPrint, dashboardSellerDuesPrintData } from '../api';
import { useSellerDues } from '../dues-hooks';

/**
 * The id of a row. The grid also asks for the id of the pinned totals row, which has no seller: it gets a
 * name of its own, so the totals row is never taken for a seller.
 */
function rowId(row: SellerDuesRow): string {
  const { party } = row as { party?: SellerDuesRow['party'] };
  return party === undefined ? 'totals' : String(party.account_id);
}

export function SellerDuesPage() {
  const navigate = useNavigate();
  const { l10n } = useLocalization();
  const address: Record<string, unknown> = useSearch({ strict: false });
  const side: SellerDuesSide = parseSellerDuesSearch(address).side ?? 'all';
  const [party, setParty] = useState<PickerValue | null>(null);
  const [focusId, setFocusId] = useState<string | null>(null);
  const partyId = party === null ? null : party.id;
  const list = useSellerDues({ side, partyId });
  // Print, PDF and Excel hold every seller of the filters on screen, in the order on screen (PRD FR-19.10).
  const print = usePrintView();
  const printInput = (language: Language, paper: Paper | null) => ({
    side,
    party_id: partyId,
    search: null,
    sort: list.sort,
    language,
    paper,
  });
  const exporter = useTableExcel('seller-dues', (sheet) => ({
    kind: 'seller_dues',
    ...printInput(sheet, null),
  }));

  const first = list.data?.pages[0];
  // A refusal shows as the state, not as the rows of the filters before it.
  const rows = list.isError ? undefined : list.data?.pages.flatMap((page) => page.rows);
  const narrowed = side !== 'all' || party !== null;

  const moveSide = useCallback(
    (next: SellerDuesSide) => {
      void navigate({
        to: '.',
        search: next === 'all' ? {} : { side: next },
        replace: true,
      });
    },
    [navigate],
  );

  const openHistory = useCallback(
    (row: SellerDuesRow) => {
      void navigate({
        to: '/bikri/party-ledgers',
        search: { side: 'seller', party: row.party.account_id },
      });
    },
    [navigate],
  );

  // The totals row is the backend's, over every seller of the filters and not over the rows loaded.
  const totals = useCallback(
    (context: ListContext) => {
      if (first === undefined) return { labelField: 'seller', values: {} };
      const word =
        first.total.side === 'zero'
          ? ''
          : context.text(first.total.side === 'dr' ? 'report-side-dr' : 'report-side-cr');
      return {
        labelField: 'seller',
        values: {
          seller: context.text('glossary-meezan'),
          payable: context.money(first.shop_owes_paisa),
          owes: context.money(first.seller_owes_paisa),
          balance: context.money(first.total.amount_paisa),
          'balance:unit': word,
        },
      };
    },
    [first],
  );

  const sideOptions: readonly { value: SellerDuesSide; label: string }[] = SELLER_DUES_SIDES.map(
    (value) => ({ value, label: l10n.getString(`seller-dues-side-${value}`) }),
  );

  const empty = narrowed ? (
    <EmptyState
      icon={SearchX}
      title={<Localized id="seller-dues-no-match" />}
      help={<Localized id="seller-dues-no-match-help" />}
      action={
        <Button
          onClick={() => {
            setParty(null);
            moveSide('all');
          }}
        >
          <Localized id="seller-dues-show-all" />
        </Button>
      }
    />
  ) : (
    <EmptyState
      icon={UserRound}
      title={<Localized id="seller-dues-empty" />}
      help={<Localized id="seller-dues-empty-help" />}
    />
  );

  if (print.preview) {
    return (
      <ListPage icon={UserRound} titleId="seller-dues-title">
        <TablePrintPreview
          exporter={exporter}
          scope={['seller_dues', side, partyId, list.sort]}
          read={(language, paper) => dashboardSellerDuesPrintData(printInput(language, paper))}
          send={(to, language, paper) =>
            dashboardSellerDuesPrint({ sheet: printInput(language, paper), to, copies: null })
          }
          onBack={print.back}
        />
      </ListPage>
    );
  }

  return (
    <ListPage
      icon={UserRound}
      titleId="seller-dues-title"
      actions={<PrintButtons onPreview={print.show} exporter={exporter} />}
      help={
        <Localized id="seller-dues-help">
          <p />
        </Localized>
      }
      filters={
        <FilterCard
          role="group"
          aria-label={l10n.getString('seller-dues-filters-label')}
          {...formNavigationProps}
          className="filter-grid"
        >
          <Field label={<Localized id="seller-dues-side-label" />}>
            {(control) => (
              <Select<SellerDuesSide>
                {...control}
                value={side}
                onChange={moveSide}
                options={sideOptions}
              />
            )}
          </Field>
          <Field label={<Localized id="seller-dues-col-seller" />}>
            {(control) => (
              <AccountPicker
                {...control}
                canAdd={false}
                allChoice
                scope="seller"
                showCard={false}
                value={party}
                onChange={(hit) => {
                  setParty(
                    hit === null
                      ? null
                      : { id: hit.id, name_ur: hit.name_ur, name_en: hit.name_en, code: hit.code },
                  );
                }}
              />
            )}
          </Field>
        </FilterCard>
      }
    >
      <ExcelExportNotices exporter={exporter} />
      <dl className="grid grid-cols-3 gap-3">
        <Fact stat label={<Localized id="dashboard-tile-payable" />}>
          {first === undefined ? null : <Num inherit paisa={first.shop_owes_paisa} />}
        </Fact>
        <Fact stat label={<Localized id="dashboard-tile-dues-sellers" />}>
          {first === undefined ? null : <Num inherit paisa={first.seller_owes_paisa} />}
        </Fact>
        <Fact stat label={<Localized id="dashboard-tile-advances" />}>
          {first === undefined ? null : <Num inherit paisa={first.advances_open_paisa} />}
        </Fact>
      </dl>
      <PagedList<SellerDuesRow>
        columns={SELLER_DUES_COLUMNS}
        rows={rows}
        getRowId={rowId}
        labelId="seller-dues-label"
        failedId="seller-dues-failed"
        error={list.isError ? uiErrorOf(list.error) : null}
        onRetry={() => {
          void list.refetch();
        }}
        pager={list.pager}
        sort={{ value: list.sort, onChange: list.setSort }}
        empty={empty}
        onOpen={openHistory}
        onFocusRow={setFocusId}
        focusRowId={focusId}
        totals={totals}
      />
    </ListPage>
  );
}
