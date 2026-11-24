// The details of the goods arrived entry that has focus in the list of a garden (PRD FR-12.40,
// FR-12.41, plan 96): its facts in three short lines, the sales that took goods from it (the first few,
// the rest on a button), and its buttons in one block: change the day to sell, change the split, write
// off, undo a write-off, cancel, and edit. It is drawn in the panel beside the list, or in a window of
// its own on a narrow screen; it is the same here. None of these buttons posts any money.
//
// Every figure is the backend's; nothing is worked out here (CLAUDE.md rule 1).

import { Localized, useLocalization } from '@fluent/react';
import { ArrowLeftRight, Ban, CalendarClock, RotateCcw } from 'lucide-react';
import { useState, type ReactNode } from 'react';
import { isoToDisplayDate } from '../../../shared/format';
import { displayName, useLanguage, useLotNames } from '../../../shared/i18n';
import {
  Button,
  CalcInfo,
  Code,
  EntryActions,
  Num,
  Table,
  type TableColumn,
} from '../../../shared/ui';
import { arrivalNumberText } from '../list-columns';
import type { ArrivalRow } from '../types';

/** The sales shown before the rest are asked for: a long history never makes the panel long. */
const LOTS_SHOWN = 4;

/** One figure of the details: a label on the start side, the figure on the end side, a rule under it. */
function Row({
  label,
  children,
  strong = false,
  wide = false,
}: {
  label: ReactNode;
  children: ReactNode;
  strong?: boolean;
  /** A label or a figure too long for half a line: the row takes the whole line. */
  wide?: boolean;
}) {
  return (
    <div
      className={`flex min-w-0 items-baseline justify-between gap-2 border-b border-border py-1 text-ink ${strong ? 'type-label' : 'type-body'} ${wide ? 'col-span-2' : ''}`}
    >
      <dt className="min-w-0 text-ink-muted">{label}</dt>
      <dd className="min-w-0 shrink-0 break-words">{children}</dd>
    </div>
  );
}

interface ArrivalDetailProps {
  arrival: ArrivalRow;
  /** A day is being reopened for the edit: the Edit button is working. */
  busy: boolean;
  /** The heading is drawn by the window the details are in (the window of a narrow screen). */
  titleless?: boolean;
  onEdit: () => void;
  onChangeSellDate: () => void;
  onChangeSplit: () => void;
  onWriteOff: () => void;
  onUndoWriteOff: () => void;
  onCancel: () => void;
}

export function ArrivalDetail({
  arrival,
  busy,
  titleless = false,
  onEdit,
  onChangeSellDate,
  onChangeSplit,
  onWriteOff,
  onUndoWriteOff,
  onCancel,
}: ArrivalDetailProps) {
  const { l10n } = useLocalization();
  const { lotItem } = useLotNames();
  const language = useLanguage();
  const [allLots, setAllLots] = useState(false);
  const say = (id: string, vars?: Record<string, string>) => l10n.getString(id, vars);
  const cancelled = arrival.status === 'cancelled';
  // The split shows only when the entry has a part for another shop (PRD FR-12.41): otherwise every
  // figure of it is the same as the figures above.
  const split = arrival.qty_other > 0 || arrival.qty_other_sold > 0;
  const lots = allLots ? arrival.lots : arrival.lots.slice(0, LOTS_SHOWN);

  // Two columns, so the list fits the panel beside the list (336 px): the sale with the name of its lot (PRD
  // FR-12.45) and its day, and the nag sold. The nag column names its size like every figure column (F18).
  const lotColumns: readonly TableColumn<ArrivalRow['lots'][number]>[] = [
    {
      id: 'sale',
      header: l10n.getString('gardens-arrival-lot-bikri'),
      cell: (lot) => (
        <span className="inline-flex flex-wrap items-baseline gap-x-2">
          <Num>{String(lot.bikri_number)}</Num>
          <span className="text-ink-muted">{lotItem(arrival.item, lot.lot_counter)}</span>
          {/* A date is never broken at its hyphens, even in a narrow panel. */}
          <span className="whitespace-nowrap">
            <Code>{isoToDisplayDate(lot.entry_date)}</Code>
          </span>
        </span>
      ),
    },
    {
      id: 'qty',
      header: l10n.getString('purchases-lot-qty-sold'),
      kind: 'number',
      size: 'qty',
      cell: (lot) => <Num>{String(lot.qty_sold)}</Num>,
    },
  ];

  const actions = cancelled ? null : (
    <div className="flex flex-wrap items-start gap-2" data-arrival-actions="">
      <EntryActions
        flags={arrival}
        noCancel
        busy={busy}
        onEdit={onEdit}
        onCancel={() => undefined}
      />
      <Button disabled={!arrival.can_change_sell_date} onClick={onChangeSellDate}>
        <CalendarClock aria-hidden="true" className="size-icon-md" />
        <Localized id="purchases-change-sell-date" />
      </Button>
      <Button disabled={!arrival.can_change_split} onClick={onChangeSplit}>
        <ArrowLeftRight aria-hidden="true" className="size-icon-md" />
        <Localized id="split-change" />
      </Button>
      <Button disabled={arrival.qty_left <= 0} onClick={onWriteOff}>
        <Ban aria-hidden="true" className="size-icon-md" />
        <Localized id="purchases-write-off" />
      </Button>
      <Button disabled={arrival.write_offs.length === 0} onClick={onUndoWriteOff}>
        <RotateCcw aria-hidden="true" className="size-icon-md" />
        <Localized id="purchases-undo-write-off" />
      </Button>
      <Button variant="danger" disabled={!arrival.can_cancel} onClick={onCancel}>
        <Ban aria-hidden="true" className="size-icon-md" />
        <Localized id="gardens-arrival-cancel" />
      </Button>
    </div>
  );

  return (
    <div className="flex min-w-0 flex-col gap-3">
      {titleless ? null : (
        <Localized
          id="gardens-arrival-card-title"
          vars={{
            number: arrivalNumberText(arrival.number, say),
            item: displayName(arrival.item, language),
          }}
        >
          <h3 className="type-section text-ink" />
        </Localized>
      )}
      <p className="type-body text-ink-muted">
        {displayName(arrival.packing, language)}
        {arrival.note === null || arrival.note === '' ? null : ` · ${arrival.note}`}
      </p>
      {cancelled ? (
        <Localized id="gardens-arrival-cancelled" vars={{ reason: arrival.cancel_reason ?? '' }}>
          <p role="status" className="type-label text-due" />
        </Localized>
      ) : null}
      {/* The buttons come first: they are what the Owner came for, and they stay in view when the facts are long. */}
      {actions}
      {/* The figures, two to a line: a label on the start side, the figure on the end side. */}
      <dl className="grid grid-cols-2 gap-x-4">
        <Row label={<Localized id="gardens-arrival-col-qty" />}>
          <Num>{String(arrival.qty)}</Num>
        </Row>
        <Row label={<Localized id="bikri-saved-qty-sold" />}>
          <Num>{String(arrival.qty_sold)}</Num>
        </Row>
        <Row label={<Localized id="bikri-stock-col-written-off" />}>
          <Num>{String(arrival.qty_written_off)}</Num>
        </Row>
        <Row label={<Localized id="gardens-arrival-col-left" />} strong>
          <Num>{String(arrival.qty_left)}</Num>
        </Row>
        <Row label={<Localized id="gardens-arrival-col-date" />}>
          <Code>{isoToDisplayDate(arrival.entry_date)}</Code>
        </Row>
        <Row label={<Localized id="bikri-stock-col-days" />}>
          <Num>{String(arrival.days_waiting)}</Num>
        </Row>
        <Row wide label={<Localized id="glossary-sell-date" />}>
          <Code>{isoToDisplayDate(arrival.sell_date)}</Code>
        </Row>
        <Row wide label={<Localized id="glossary-commission" />}>
          <span className="inline-flex items-center gap-1">
            <Num paisa={arrival.commission_paisa} />
            <CalcInfo working={arrival.commission_working} inCell />
          </span>
        </Row>
        {split ? (
          <>
            <Row wide label={<Localized id="goods-line-col-shop" />}>
              {arrival.shop_name ?? ''}
            </Row>
            <Row wide label={<Localized id="purchases-detail-qty-other" />}>
              <Num>{String(arrival.qty_other)}</Num>
            </Row>
            <Row wide label={<Localized id="purchases-detail-our-left" />}>
              <Num>{String(arrival.qty_our_left)}</Num>
            </Row>
            <Row wide label={<Localized id="purchases-detail-other-left" />}>
              <Num>{String(arrival.qty_other_left)}</Num>
            </Row>
            <Row wide label={<Localized id="purchases-detail-our-sold" />}>
              <Num>{String(arrival.qty_our_sold)}</Num>
            </Row>
            <Row wide label={<Localized id="purchases-detail-other-sold" />}>
              <Num>{String(arrival.qty_other_sold)}</Num>
            </Row>
          </>
        ) : null}
      </dl>
      {arrival.lots.length === 0 ? null : (
        <div className="flex min-w-0 flex-col gap-2">
          <Table
            dense
            columns={lotColumns}
            rows={lots}
            rowKey={(lot) => String(lot.lot_id)}
            label={l10n.getString('gardens-arrival-lots')}
          />
          {arrival.lots.length <= LOTS_SHOWN ? null : (
            <div>
              <Button
                small
                expanded={allLots}
                onClick={() => {
                  setAllLots(!allLots);
                }}
              >
                <Localized
                  id={allLots ? 'gardens-arrival-lots-less' : 'gardens-arrival-lots-more'}
                  vars={{ count: String(arrival.lots.length) }}
                />
              </Button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
