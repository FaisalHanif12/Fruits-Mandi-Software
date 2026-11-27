// One garden (route /gardens/$id; PRD 12.11, FR-7.67, FR-12.34 to FR-12.38): the facts of the
// garden with its running balance, the entry form with its type list (قسط, باغ خرچ, دوسری دکان میں
// فروخت), each partner's money in and back, the history of the garden with the balance after every
// row, and, while it is open, the closing.
//
// A closed garden takes no new entry, no lot and no fruit (FR-12.38): the form is replaced by a
// note that says so, with a button that undoes the closing while its day is open. Everything else
// stays readable.
//
// The page stays organized however many entries the garden has (plan 96, PRD FR-12.53): the facts with
// the four key figures at the top, the entry form, the goods arrived in two lists (not sold yet, then
// sold) with the details of an entry under its list, where the sale money is, the history, then the
// garden's figures (open; each partner's money in them). A saved goods arrived entry shows at the head
// of the first list, tinted, and the page does not move.
//
// Four states: loading, an error with a retry (also for a garden that does not exist), the garden.

import { Localized } from '@fluent/react';
import { useParams, useSearch } from '@tanstack/react-router';
import { Undo2 } from 'lucide-react';
import { useState } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate } from '../../../shared/format';
import { Button, ErrorState, LoadingState, Notice } from '../../../shared/ui';
import type { PagerState } from '../../../shared/hooks';
import { CloseGardenCard } from '../components/CloseGardenCard';
import { GardenArrivalEdit } from '../components/GardenArrivalEdit';
import { GardenArrivalsSection, type AddedArrivals } from '../components/GardenArrivalsSection';
import { GardenEntryEdit } from '../components/GardenEntryEdit';
import { GardenEntryEditor } from '../components/GardenEntryEditor';
import { GardenFiguresFold, GardenHeaderCard } from '../components/GardenHeaderCard';
import { GardenHeaderEdit } from '../components/GardenHeaderEdit';
import { GardenHistorySection } from '../components/GardenHistorySection';
import { GardenSaleMoneySection } from '../components/GardenSaleMoneySection';
import { ReopenGardenDialog } from '../components/ReopenGardenDialog';
import { useGarden, type GardenView } from '../hooks';
import { parseGardenSearch } from '../screen-type';
import type { ArrivalRow, GardenHistoryRow } from '../types';

/** A positive whole number from the address, or nothing. */
function positiveId(value: unknown): number | null {
  const id = Number(value);
  return Number.isSafeInteger(id) && id > 0 ? id : null;
}

function GardenBody({
  view,
  today,
  pager,
}: {
  view: GardenView;
  today: string;
  pager: PagerState;
}) {
  // What the address asks the screen to open on (PRD FR-7.79): an entry type, or the closing.
  const rawSearch: Record<string, unknown> = useSearch({ strict: false });
  const openType = parseGardenSearch(rawSearch).type;
  const [reopening, setReopening] = useState(false);
  // The entry open for edit (plan 44): the form of a new entry gives way to its edit form.
  const [editing, setEditing] = useState<GardenHistoryRow | null>(null);
  const [editingArrival, setEditingArrival] = useState<ArrivalRow | null>(null);
  // The facts of the garden open for edit (names, main fruit, rent, season end, note).
  const [editingHeader, setEditingHeader] = useState(false);
  // The goods arrived entries the form has just saved: the list shows them first and tinted.
  const [added, setAdded] = useState<AddedArrivals | null>(null);
  const { detail } = view;
  const garden = detail.row;
  const open = garden.status === 'open';
  const editArrival = (arrival: ArrivalRow) => {
    setEditing(null);
    setEditingHeader(false);
    setEditingArrival(arrival);
  };
  return (
    <div className="flex max-w-page flex-col gap-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <Localized id="gardens-title">
          <h1 className="sr-only normal:not-sr-only normal:type-title" />
        </Localized>
      </div>
      <GardenHeaderCard
        garden={garden}
        onEdit={() => {
          setEditing(null);
          setEditingArrival(null);
          setEditingHeader(true);
        }}
      />
      {editingHeader ? (
        <GardenHeaderEdit
          key={String(garden.version)}
          garden={garden}
          onDone={() => {
            setEditingHeader(false);
          }}
        />
      ) : editingArrival !== null ? (
        <GardenArrivalEdit
          key={`${String(editingArrival.id)}-${String(editingArrival.version)}`}
          garden={garden}
          arrival={editingArrival}
          onDone={() => {
            setEditingArrival(null);
          }}
        />
      ) : editing !== null ? (
        <GardenEntryEdit
          key={`${String(editing.entry_id)}-${String(editing.version)}`}
          garden={garden}
          row={editing}
          onDone={() => {
            setEditing(null);
          }}
        />
      ) : open ? (
        <GardenEntryEditor
          garden={garden}
          today={today}
          initialType={openType === undefined || openType === 'close' ? undefined : openType}
          onArrivalsSaved={(ids) => {
            setAdded((previous) => ({ ids, token: (previous?.token ?? 0) + 1 }));
          }}
        />
      ) : (
        <Notice
          tone="info"
          action={
            <Button
              onClick={() => {
                setReopening(true);
              }}
            >
              <Undo2 aria-hidden="true" className="size-icon-md" />
              <Localized id="gardens-reopen-button" />
            </Button>
          }
        >
          <Localized
            id="gardens-closed-notice"
            vars={{
              date: garden.closed_date === null ? '' : isoToDisplayDate(garden.closed_date),
            }}
          >
            <span />
          </Localized>
        </Notice>
      )}
      <GardenArrivalsSection garden={garden} stock="in_stock" added={added} onEdit={editArrival} />
      <GardenArrivalsSection garden={garden} stock="sold_out" onEdit={editArrival} />
      <GardenSaleMoneySection garden={garden} />
      <GardenHistorySection
        history={view.history}
        count={view.historyCount}
        pager={pager}
        onEdit={(row) => {
          setEditingArrival(null);
          setEditingHeader(false);
          setEditing(row);
        }}
      />
      <GardenFiguresFold
        garden={garden}
        results={detail.partner_results}
        partnersLocked={detail.partners_locked}
        closed={!open}
      />
      {open ? (
        <CloseGardenCard garden={garden} today={today} focusOnOpen={openType === 'close'} />
      ) : null}
      <ReopenGardenDialog garden={garden} open={reopening} onOpenChange={setReopening} />
    </div>
  );
}

/** The route of one garden: the garden comes from the address. */
export function GardenPage() {
  const { id: rawId } = useParams({ strict: false });
  return <GardenScreen id={positiveId(rawId)} />;
}

/** One garden by its id, with its four states. The development page shows it without a route. */
export function GardenScreen({ id }: { id: number | null }) {
  const day = useDayStatus();
  const garden = useGarden(id);
  if (garden.isPending || day.isPending) return <LoadingState />;
  if (garden.isError || day.isError || garden.data === undefined) {
    const failure: unknown = garden.isError ? garden.error : day.error;
    return (
      <ErrorState
        title={<Localized id="gardens-detail-failed" />}
        error={uiErrorOf(failure)}
        onRetry={() => {
          void garden.refetch();
          void day.refetch();
        }}
      />
    );
  }
  // A new key for each garden of the address, so the screen starts on the one it names.
  return (
    <GardenBody key={id ?? 'none'} view={garden.data} today={day.data.today} pager={garden.pager} />
  );
}
