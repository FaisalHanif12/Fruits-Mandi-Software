// A later sale (route /bikri/later?lot=40; PRD 12.9, FR-12.17): the goods left in a lot are sold on
// a later day as a posting of their own. The lot is shown read-only; only new sale lines are typed,
// dated the day of the sale (an open day, not before the goods were received). The first posting
// and its day are not touched, and the lot's freight, labour and the rest are not charged again.
// The backend works the new lines out with the rules of the later day and sends the live summary
// and the partner split; the screen shows them. Four states: loading, an error with a retry, no
// lot (a button back to the list) and the form.

import { Localized } from '@fluent/react';
import { useNavigate, useSearch } from '@tanstack/react-router';
import { PackageX } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import type { StockRow } from '../../../bindings/StockRow';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate } from '../../../shared/format';
import {
  Button,
  Card,
  DateInput,
  EmptyState,
  ErrorState,
  Field,
  LoadingState,
  Notice,
  fieldError,
} from '../../../shared/ui';
import { LinesForm } from '../components/LinesForm';
import { LotFacts } from '../components/stock/StockLotCard';
import { BikriDraftProvider, createBikriDraft, type BikriDraftStores } from '../draft-store';
import { useStockList } from '../hooks/useBikriReads';
import { useSaveLaterSale } from '../hooks/useSaveBikri';
import { laterSaleSource } from '../preview-source';
import { LOT, ON } from '../types';

/** All the lots, also the ones with nothing left, so a lot that was sold out meanwhile is found. */
const ANY_LOT = {
  date: null,
  from: null,
  to: null,
  seller_id: null,
  item_id: null,
  include_written_off: true,
} as const;

/** The key of the one lot row. The preview's answer is matched to it by place. */
function lotKeyOf(lotId: number): string {
  return `lot:${String(lotId)}`;
}

/** The stores of the form: one lot row for the lot, one blank line, today as the day. */
function createLaterSaleStores(row: StockRow, today: string): BikriDraftStores {
  const stores = createBikriDraft(isoToDisplayDate(today));
  const key = lotKeyOf(row.lot_id);
  stores.lots.setState({
    rows: [
      {
        rowId: key,
        // The seller and what the lot holds are already known: nothing is filled again from his
        // last lot (PRD FR-7.51).
        typed: {
          [LOT.lotId]: String(row.lot_id),
          [LOT.sellerFilled]: ON,
          [LOT.memoryFilledFor]: String(row.seller.id),
        },
        picked: {
          [LOT.seller]: row.seller,
          [LOT.item]: row.item,
          [LOT.packing]: row.packing,
        },
      },
    ],
  });
  stores.view.getState().select(key);
  return stores;
}

function LaterSaleForm({ row, today }: { row: StockRow; today: string }) {
  const navigate = useNavigate();
  const [stores] = useState(() => createLaterSaleStores(row, today));
  const lotKey = lotKeyOf(row.lot_id);
  const mutation = useSaveLaterSale(row.lot_id);
  const date = stores.header((state) => state.entryDate);
  // After a save the form goes back to the list. The guard is told first, so it does not ask.
  const [saved, setSaved] = useState(false);
  useEffect(() => {
    if (saved) void navigate({ to: '/bikri/stock' });
  }, [saved, navigate]);

  return (
    <BikriDraftProvider value={stores}>
      <LinesForm
        lotKey={lotKey}
        source={laterSaleSource(row.lot_id, lotKey)}
        mutation={mutation}
        toastId="bikri-later-saved-toast"
        onSaved={() => {
          setSaved(true);
        }}
        title={
          <Localized id="bikri-later-title" vars={{ number: String(row.bikri_number) }}>
            <span />
          </Localized>
        }
        help={<Localized id="bikri-later-help" />}
        dayField={(error) => (
          <Field label={<Localized id="bikri-later-date" />} error={fieldError(error, 'date')}>
            {(control) => (
              <DateInput
                {...control}
                value={date}
                onChange={(entryDate) => {
                  stores.header.getState().patch({ entryDate });
                }}
              />
            )}
          </Field>
        )}
        dirty={saved ? false : undefined}
        saveId="bikri-later-save"
        canSave
        onBack={() => {
          void navigate({ to: '/bikri/stock' });
        }}
      >
        <Card compact title={<Localized id="bikri-later-lot-title" />}>
          <LotFacts row={row} />
          <Notice tone="info">
            <Localized id="bikri-later-no-lot-charges" />
          </Notice>
        </Card>
      </LinesForm>
    </BikriDraftProvider>
  );
}

export function LaterSalePage() {
  const navigate = useNavigate();
  const search = useSearch({ strict: false });
  const lotId = (search as { lot?: number }).lot ?? null;
  const day = useDayStatus();
  const list = useStockList(ANY_LOT);
  // The lot as it was when the form opened. A save reads the list again and the lot may be gone
  // from it; the form must not turn into "nothing left" before it has gone back (plan 16).
  const [kept, setKept] = useState<StockRow | null>(null);
  const found = list.data?.rows.find((candidate) => candidate.lot_id === lotId);
  if (kept === null && found !== undefined && found.balance > 0) setKept(found);

  const toList = (
    <Button
      variant="primary"
      onClick={() => {
        void navigate({ to: '/bikri/stock' });
      }}
    >
      <Localized id="bikri-later-to-stock" />
    </Button>
  );

  if (lotId === null) {
    return (
      <EmptyState
        icon={PackageX}
        title={<Localized id="bikri-later-no-lot" />}
        help={<Localized id="bikri-later-no-lot-help" />}
        action={toList}
      />
    );
  }
  if (day.isError || list.isError) {
    return (
      <ErrorState
        title={<Localized id="bikri-later-failed" />}
        error={uiErrorOf(day.isError ? day.error : list.error)}
        onRetry={() => {
          void list.refetch();
          void day.refetch();
        }}
      />
    );
  }
  if (day.isPending || list.isPending) return <LoadingState />;
  if (kept !== null) {
    return <LaterSaleForm key={kept.lot_id} row={kept} today={day.data.today} />;
  }
  const row = found;
  if (row === undefined) {
    return (
      <EmptyState
        icon={PackageX}
        title={<Localized id="bikri-later-lot-missing" />}
        help={<Localized id="bikri-later-lot-missing-help" />}
        action={toList}
      />
    );
  }
  if (row.balance <= 0) {
    return (
      <EmptyState
        icon={PackageX}
        title={<Localized id="bikri-later-nothing-left" />}
        help={<Localized id="bikri-later-nothing-left-help" />}
        action={toList}
      />
    );
  }
  return null;
}
