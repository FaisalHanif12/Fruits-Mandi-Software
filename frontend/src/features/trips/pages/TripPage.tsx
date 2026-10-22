// One trip (route /trips/$id; PRD 12.10, FR-7.67, FR-12.25, FR-12.27, FR-12.33): the facts of the
// trip, the entry form with its type list (خریداری, سفر خرچ, دوسری دکان میں فروخت), the trip's
// purchases with their stock and result, each partner's money in and back, and the history of
// the trip. `?purchase=7` shows purchase 7 in the card under the list.
//
// A trip closes by itself when it has a purchase and nothing is left, and opens again if a sale
// or a write-off of it is reversed (FR-12.33). A closed trip takes no new entry: the form is
// replaced by a note that says so. Everything else stays readable.
//
// Four states: loading, an error with a retry (also for a trip that does not exist), the trip.

import { Localized } from '@fluent/react';
import { useNavigate, useParams, useSearch } from '@tanstack/react-router';
import { ArrowLeft } from 'lucide-react';
import { useState } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import { uiErrorOf } from '../../../shared/api';
import { Button, ErrorState, LoadingState, Notice } from '../../../shared/ui';
import { TripEntriesSection } from '../components/TripEntriesSection';
import { TripEntryEditor } from '../components/TripEntryEditor';
import { TripFiguresCard, TripHeaderCard } from '../components/TripHeaderCard';
import { TripPartnersCard } from '../components/TripPartnersCard';
import { TripPurchasesSection } from '../components/TripPurchasesSection';
import { useTrip, type TripView } from '../hooks';

/** A positive whole number from the address, or nothing. */
function positiveId(value: unknown): number | null {
  const id = Number(value);
  return Number.isSafeInteger(id) && id > 0 ? id : null;
}

function TripBody({
  view,
  today,
  initialPurchase,
  hasMore,
  loadingMore,
  onLoadMore,
}: {
  view: TripView;
  today: string;
  initialPurchase: number | null;
  hasMore: boolean;
  loadingMore: boolean;
  onLoadMore: () => void;
}) {
  const navigate = useNavigate();
  const { detail } = view;
  const trip = detail.row;
  const [selectedPurchase, setSelectedPurchase] = useState<number | null>(initialPurchase);
  return (
    <div className="flex max-w-page flex-col gap-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <Localized id="trips-title">
          <h1 className="sr-only normal:not-sr-only normal:type-title" />
        </Localized>
        <Button
          onClick={() => {
            void navigate({ to: '/trips' });
          }}
        >
          <ArrowLeft aria-hidden="true" className="size-icon-md rtl:rotate-180" />
          <Localized id="trips-back" />
        </Button>
      </div>
      <TripHeaderCard trip={trip} />
      {trip.status === 'open' ? (
        <TripEntryEditor
          trip={trip}
          purchases={detail.purchases}
          today={today}
          onPurchaseSaved={setSelectedPurchase}
        />
      ) : (
        <Notice tone="info">
          <Localized id="trips-closed-notice" />
        </Notice>
      )}
      <TripFiguresCard trip={trip} />
      <TripPurchasesSection
        purchases={detail.purchases}
        selectedId={selectedPurchase}
        onSelect={setSelectedPurchase}
      />
      <TripPartnersCard results={detail.partner_results} />
      <TripEntriesSection
        entries={view.entries}
        count={view.entriesCount}
        hasMore={hasMore}
        loadingMore={loadingMore}
        onLoadMore={onLoadMore}
        purchases={detail.purchases}
      />
    </div>
  );
}

/** The route of one trip: the trip and the chosen purchase come from the address. */
export function TripPage() {
  const { id: rawId } = useParams({ strict: false });
  const rawSearch: Record<string, unknown> = useSearch({ strict: false });
  return <TripScreen id={positiveId(rawId)} purchase={positiveId(rawSearch['purchase'])} />;
}

/** One trip by its id, with its four states. The development page shows it without a route. */
export function TripScreen({ id, purchase }: { id: number | null; purchase: number | null }) {
  const day = useDayStatus();
  const trip = useTrip(id);
  if (trip.isPending || day.isPending) return <LoadingState />;
  if (trip.isError || day.isError || trip.data === undefined) {
    const failure: unknown = trip.isError ? trip.error : day.error;
    return (
      <ErrorState
        title={<Localized id="trips-detail-failed" />}
        error={uiErrorOf(failure)}
        onRetry={() => {
          void trip.refetch();
          void day.refetch();
        }}
      />
    );
  }
  // A new key for each trip of the address, so the screen starts on the one it names.
  return (
    <TripBody
      key={id ?? 'none'}
      view={trip.data}
      today={day.data.today}
      initialPurchase={purchase}
      hasMore={trip.hasNextPage}
      loadingMore={trip.isFetchingNextPage}
      onLoadMore={() => {
        void trip.fetchNextPage();
      }}
    />
  );
}
