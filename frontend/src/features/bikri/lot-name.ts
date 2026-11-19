// The name of a lot as words, for every label that points at a lot (PRD FR-12.6, FR-12.45, plan 71 L1
// to L4, L8): the seller, the item and the counter, for example "Ehsan, Mango-2" or "احسان، آم-2".
// The counter belongs to one seller and one item and runs for the seller's whole life: the backend
// gives it (a saved lot has it; a new lot shows the number it will get when it is saved, worked out
// from the saved lots and the lots above it in the draft), so the screen only draws it. The seller
// and the item come through `displayName` (with the tag, Code Standards F38). The words that join
// them (the comma, the dash) are in Fluent (`bikri-lot-name`), so Urdu and English each use their
// own, and each part is isolated by Fluent, so the counter never flips inside Urdu text (F25).

import { createContext, useCallback, useContext, useMemo } from 'react';
import type { SavedLot } from '../../bindings/SavedLot';
import type { BikriPreview } from '../../bindings/BikriPreview';
import type { EntryRow } from '../../shared/grid';
import { useLotNames } from '../../shared/i18n';
import { LOT } from './types';

/** What the Fluent messages of a lot label take: its number, its name and whether it has one. */
export interface LotLabelVars {
  lot: string;
  who: string;
  /** `yes` when the lot has a name to show; the message then adds it after the number. */
  named: 'yes' | 'no';
}

/**
 * The counters the backend gave the lots of the screen, by the key of the lot row, each with the
 * seller and the item it was worked out for. A counter is shown only while the row still has that
 * seller and that item, so a lot whose seller was just changed shows his name alone until the new
 * answer comes, never the old seller's counter on the new seller.
 */
export type LotCounters = ReadonlyMap<
  string,
  { sellerId: number | null; itemId: number | null; counter: number }
>;

const NO_COUNTERS: LotCounters = new Map();

/** The counters of an answer of the preview, for the lot rows as they are now. */
export function lotCountersOf(
  preview: BikriPreview | undefined,
  rows: readonly EntryRow[],
): LotCounters {
  if (preview === undefined) return NO_COUNTERS;
  const byKey = new Map(rows.map((row) => [row.rowId, row]));
  const made = new Map<
    string,
    { sellerId: number | null; itemId: number | null; counter: number }
  >();
  for (const lot of preview.lots) {
    const row = byKey.get(lot.key);
    if (row === undefined || lot.lot_counter === null) continue;
    made.set(lot.key, {
      sellerId: row.picked?.[LOT.seller]?.id ?? null,
      itemId: row.picked?.[LOT.item]?.id ?? null,
      counter: lot.lot_counter,
    });
  }
  return made;
}

const LotCountersContext = createContext<LotCounters>(NO_COUNTERS);

/** Gives the labels below the screen the counters of its lots (`useLotWho` reads them). */
export const LotCountersProvider = LotCountersContext.Provider;

/**
 * The counters of the lots of the screen from the last answer of the preview. They are read from the
 * lot rows when the answer arrives, so the seller and the item each counter belongs to are known.
 */
export function useLotCounters(
  preview: BikriPreview | undefined,
  getRows: () => readonly EntryRow[],
): LotCounters {
  // The rows are read once for each answer, not on every render: an answer is the moment the counter
  // and the row it belongs to agree.
  return useMemo(() => lotCountersOf(preview, getRows()), [preview, getRows]);
}

/**
 * The words that name a lot of the screen: "Ehsan, Mango-2". It is the empty text while neither the
 * seller nor the item is chosen, and then a label shows the number alone. The counter is the
 * backend's (`LotPreview.lot_counter`), from the counters the screen holds (an argument, or the ones
 * the page above gave through `LotCountersProvider`).
 */
export function useLotWho(counters?: LotCounters): (row: EntryRow | undefined) => string {
  const inherited = useContext(LotCountersContext);
  const { lotName: nameText } = useLotNames();
  const known = counters ?? inherited;
  return useCallback(
    (row) => {
      if (row === undefined) return '';
      const seller = row.picked?.[LOT.seller];
      const item = row.picked?.[LOT.item];
      const entry = known.get(row.rowId);
      if (entry === undefined) return nameText(seller, item, null);
      // The counter is shown only for the seller and the item it was worked out for.
      const same = entry.sellerId === (seller?.id ?? null) && entry.itemId === (item?.id ?? null);
      const counter = same ? entry.counter : null;
      return nameText(seller, item, counter);
    },
    [known, nameText],
  );
}

/** The values of a lot label: the lot's number and its name (empty text: the number stands alone). */
export function lotLabelVars(number: number, who: string): LotLabelVars {
  return { lot: String(number), who, named: who === '' ? 'no' : 'yes' };
}

/**
 * The same words for a saved lot: its seller, its item and the counter it was given when it was
 * first saved (PRD FR-12.45). Every saved lot has all three.
 */
export function useSavedLotWho(): (
  lot: Pick<SavedLot, 'seller' | 'item' | 'lot_counter'>,
) => string {
  const { lotName } = useLotNames();
  return useCallback((lot) => lotName(lot.seller, lot.item, lot.lot_counter), [lotName]);
}
