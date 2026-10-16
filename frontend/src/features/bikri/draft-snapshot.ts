// The draft of the بکری as text (PRD FR-12.10, NFR-REL-2). The screen sends it to the backend every
// 5 seconds while the user types, and after a crash or a power cut the app offers it back. The
// snapshot is the draft stores as they are: every value the user typed, as text, and the records
// he picked, with both names. It holds no money worked out and no answer of the preview, so
// opening it asks the backend for the numbers again (CLAUDE.md rule 1).
//
// A snapshot that is not ours (another version, damaged text) is refused, never half used.

import type { EntryRow, PickedRecord } from '../../shared/grid';
import type { BikriDraftStores, HeaderState } from './draft-store';

/** Raised when the format of the snapshot changes, so an old one is never read wrongly. */
const SNAPSHOT_VERSION = 1;

/** The screen key of the بکری draft in the backend (`drafts_save`). */
export const BIKRI_DRAFT_SCREEN = 'bikri';

export interface DraftSnapshot {
  v: typeof SNAPSHOT_VERSION;
  header: HeaderState;
  lots: EntryRow[];
  /** The sale lines of each lot, by the row id of the lot. */
  sales: Record<string, EntryRow[]>;
  selectedLot: string | null;
  wholeView: boolean;
}

/** The draft stores as text, ready for `drafts_save`. */
export function serialiseDraft(stores: BikriDraftStores): string {
  const lots = stores.lots.getState().rows;
  const sales: Record<string, EntryRow[]> = {};
  for (const lot of lots) sales[lot.rowId] = stores.salesFor(lot.rowId).getState().rows;
  // The header store also holds its `patch` function; JSON leaves a function out.
  const header: HeaderState = stores.header.getState();
  const view = stores.view.getState();
  const snapshot: DraftSnapshot = {
    v: SNAPSHOT_VERSION,
    header,
    lots,
    sales,
    selectedLot: view.selectedLot,
    wholeView: view.wholeView,
  };
  return JSON.stringify(snapshot);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function isText(value: unknown): value is string {
  return typeof value === 'string';
}

function isPicked(value: unknown): value is PickedRecord {
  return (
    isRecord(value) &&
    typeof value['id'] === 'number' &&
    isText(value['code']) &&
    isText(value['name_ur']) &&
    isText(value['name_en'])
  );
}

function isPickedOrNull(value: unknown): value is PickedRecord | null {
  return value === null || isPicked(value);
}

function isRow(value: unknown): value is EntryRow {
  if (!isRecord(value) || !isText(value['rowId']) || !isRecord(value['typed'])) return false;
  if (!Object.values(value['typed']).every(isText)) return false;
  const picked = value['picked'];
  return picked === undefined || (isRecord(picked) && Object.values(picked).every(isPicked));
}

function isRows(value: unknown): value is EntryRow[] {
  return Array.isArray(value) && value.length > 0 && value.every(isRow);
}

function isHeader(value: unknown): value is HeaderState {
  return (
    isRecord(value) &&
    [
      'entryDate',
      'billDate',
      'book',
      'truckNo',
      'driverName',
      'driverPhone',
      'totalFreight',
      'note',
    ].every((key) => isText(value[key])) &&
    isPickedOrNull(value['zamindar']) &&
    isPickedOrNull(value['adda']) &&
    isPickedOrNull(value['store'])
  );
}

/** The snapshot in `text`, or null when it is not a valid snapshot of this version. */
export function parseSnapshot(text: string): DraftSnapshot | null {
  let value: unknown;
  try {
    value = JSON.parse(text);
  } catch {
    return null;
  }
  if (!isRecord(value) || value['v'] !== SNAPSHOT_VERSION) return null;
  const { header, lots, sales, selectedLot, wholeView } = value;
  if (!isHeader(header) || !isRows(lots) || !isRecord(sales)) return null;
  const salesByLot: Record<string, EntryRow[]> = {};
  for (const lot of lots) {
    const lines = sales[lot.rowId];
    if (!isRows(lines)) return null;
    salesByLot[lot.rowId] = lines;
  }
  if (selectedLot !== null && !isText(selectedLot)) return null;
  if (typeof wholeView !== 'boolean') return null;
  return { v: SNAPSHOT_VERSION, header, lots, sales: salesByLot, selectedLot, wholeView };
}

/** Puts a snapshot into new stores, so the screen opens exactly where the user stopped. */
export function applySnapshot(stores: BikriDraftStores, snapshot: DraftSnapshot): void {
  stores.header.setState(snapshot.header);
  stores.lots.setState({ rows: snapshot.lots });
  for (const lot of snapshot.lots) {
    stores.salesFor(lot.rowId).setState({ rows: snapshot.sales[lot.rowId] ?? [] });
  }
  const selected =
    snapshot.selectedLot !== null && snapshot.lots.some((lot) => lot.rowId === snapshot.selectedLot)
      ? snapshot.selectedLot
      : (snapshot.lots[0]?.rowId ?? null);
  stores.view.getState().select(selected);
  stores.view.getState().setWhole(snapshot.wholeView);
}
