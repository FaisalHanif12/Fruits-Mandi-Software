// A lot of a garden's goods arrived (PRD FR-12.40, plan 33 v1.1). When the seller of a lot is a
// garden and the lot takes its goods from one goods arrived entry of that garden, this panel sits
// under the lots grid for the selected lot. It says where the goods come from (the garden and the
// entry) and how many the entry can still give this lot, from the backend's preview
// (`LotPreview.arrival`): that is what the entry has left after the lots above it in this form.
//
// - The item, the packing and the quantity are in the lot's cells, filled from the entry like
//   typed values (NFR-USE-27, Code Standards F43). The item and packing follow the entry, so they
//   are shown and not typed on this lot.
// - The lot must sell exactly the quantity it takes (ERR_ARRIVAL_LOT_NOT_FULLY_SOLD, shown at the
//   quantity cell); the rest stays in the entry for the next new بکری.
// - Nothing is worked out here. The numbers are the backend's (Code Standards F42).

import { Localized } from '@fluent/react';
import { Sprout } from 'lucide-react';
import type { LotPreview } from '../../../bindings/LotPreview';
import type { UiError } from '../../../bindings/UiError';
import type { EntryRow } from '../../../shared/grid';
import { displayName, useLanguage } from '../../../shared/i18n';
import { Card, Code, Fact, Num, fieldError } from '../../../shared/ui';
import { LOT } from '../types';

interface ArrivalLotPanelProps {
  row: EntryRow;
  lotIndex: number;
  error: UiError | null;
  /** The backend's answer for this lot, or undefined while it cannot be worked out. */
  lot: LotPreview | undefined;
}

export function ArrivalLotPanel({ row, lotIndex, error, lot }: ArrivalLotPanelProps) {
  const language = useLanguage();
  const garden = row.picked?.[LOT.seller];
  const arrival = lot?.arrival ?? null;
  const problem = fieldError(error, `lots[${String(lotIndex)}].arrival_id`);
  return (
    <Card
      compact
      title={
        <span className="inline-flex items-center gap-2">
          <Sprout aria-hidden="true" className="size-icon-md" />
          <Localized id="bikri-arrival-title" />
        </span>
      }
    >
      <div data-arrival-lot-for={row.rowId} className="flex flex-col gap-3">
        <Localized id="bikri-arrival-hint">
          <p className="type-body text-ink-muted" />
        </Localized>
        <dl className="grid grid-cols-3 gap-x-4 gap-y-3 normal:grid-cols-4">
          <Fact label={<Localized id="glossary-bagh" />}>
            {garden === undefined ? null : (
              <span className="inline-flex flex-wrap items-center gap-2">
                <Code>{garden.code}</Code>
                <span>{displayName(garden, language)}</span>
              </span>
            )}
          </Fact>
          {arrival === null ? null : (
            <>
              <Fact label={<Localized id="glossary-maal-aaya" />}>
                <Num>{String(arrival.arrival_number)}</Num>
              </Fact>
              <Fact label={<Localized id="bikri-arrival-left" />}>
                <Num>{String(arrival.qty_left)}</Num>
              </Fact>
            </>
          )}
        </dl>
        {problem === null ? null : <div role="alert">{problem}</div>}
      </div>
    </Card>
  );
}
