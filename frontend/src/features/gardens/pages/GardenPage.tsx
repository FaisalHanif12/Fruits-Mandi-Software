// One garden (route /gardens/$id; PRD 12.11, FR-7.67, FR-12.34 to FR-12.38): the facts of the
// garden with its running balance, the entry form with its type list (قسط, باغ خرچ, دوسری دکان میں
// فروخت), each partner's money in and back, the history of the garden with the balance after every
// row, and, while it is open, the closing.
//
// A closed garden takes no new entry, no lot and no fruit (FR-12.38): the form is replaced by a
// note that says so, with a button that undoes the closing while its day is open. Everything else
// stays readable.
//
// Four states: loading, an error with a retry (also for a garden that does not exist), the garden.

import { Localized } from '@fluent/react';
import { useNavigate, useParams } from '@tanstack/react-router';
import { ArrowLeft, Undo2 } from 'lucide-react';
import { useState } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate } from '../../../shared/format';
import { Button, ErrorState, LoadingState, Notice } from '../../../shared/ui';
import { CloseGardenCard } from '../components/CloseGardenCard';
import { GardenArrivalsSection } from '../components/GardenArrivalsSection';
import { GardenEntryEditor } from '../components/GardenEntryEditor';
import { GardenFiguresCard, GardenHeaderCard } from '../components/GardenHeaderCard';
import { GardenHistorySection } from '../components/GardenHistorySection';
import { GardenPartnersCard } from '../components/GardenPartnersCard';
import { ReopenGardenDialog } from '../components/ReopenGardenDialog';
import { useGarden, type GardenView } from '../hooks';

/** A positive whole number from the address, or nothing. */
function positiveId(value: unknown): number | null {
  const id = Number(value);
  return Number.isSafeInteger(id) && id > 0 ? id : null;
}

function GardenBody({
  view,
  today,
  hasMore,
  loadingMore,
  onLoadMore,
}: {
  view: GardenView;
  today: string;
  hasMore: boolean;
  loadingMore: boolean;
  onLoadMore: () => void;
}) {
  const navigate = useNavigate();
  const [reopening, setReopening] = useState(false);
  const { detail } = view;
  const garden = detail.row;
  const open = garden.status === 'open';
  return (
    <div className="flex max-w-page flex-col gap-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <Localized id="gardens-title">
          <h1 className="sr-only normal:not-sr-only normal:type-title" />
        </Localized>
        <Button
          onClick={() => {
            void navigate({ to: '/gardens' });
          }}
        >
          <ArrowLeft aria-hidden="true" className="size-icon-md rtl:rotate-180" />
          <Localized id="gardens-back" />
        </Button>
      </div>
      <GardenHeaderCard garden={garden} />
      {open ? (
        <GardenEntryEditor garden={garden} today={today} />
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
      <GardenArrivalsSection garden={garden} />
      <GardenFiguresCard garden={garden} />
      <GardenPartnersCard results={detail.partner_results} closed={!open} />
      <GardenHistorySection
        history={view.history}
        count={view.historyCount}
        hasMore={hasMore}
        loadingMore={loadingMore}
        onLoadMore={onLoadMore}
      />
      {open ? <CloseGardenCard garden={garden} today={today} /> : null}
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
    <GardenBody
      key={id ?? 'none'}
      view={garden.data}
      today={day.data.today}
      hasMore={garden.hasNextPage}
      loadingMore={garden.isFetchingNextPage}
      onLoadMore={() => {
        void garden.fetchNextPage();
      }}
    />
  );
}
