// The draft of one بکری, kept above the screen (Code Standards F39, entry-grid skill). A language
// switch rebuilds the grids and loses nothing, because everything the user typed is here: the
// header in one store, the lots in the lots grid's store, and the sale lines of each lot in a
// store of their own (the sale lines grid shows the lines of the selected lot, PRD FR-12.1).
//
// Any change in any of them bumps one counter, so the screen builds the draft for the preview
// again (Code Standards F35). Nothing here works out money: every value is text, as typed.

import { createContext, useContext } from 'react';
import { create, type StoreApi, type UseBoundStore } from 'zustand';
import { createEntryStore, type EntryStore, type PickedRecord } from '../../shared/grid';
import { createLinesUnion } from './union-store';

/** The header of PRD 12.2 as typed. The pickers hold the record chosen. */
export interface HeaderState {
  entryDate: string;
  billDate: string;
  book: string;
  truckNo: string;
  zamindar: PickedRecord | null;
  adda: PickedRecord | null;
  driverName: string;
  driverPhone: string;
  totalFreight: string;
  store: PickedRecord | null;
  note: string;
}

interface HeaderStore extends HeaderState {
  patch: (patch: Partial<HeaderState>) => void;
}

/** What the screen shows besides the typed values: which lot, and which parts are open. */
interface ViewState {
  /** The lot whose sale lines the grid shows. Null until the first lot row exists. */
  selectedLot: string | null;
  /** "The whole بکری" in the summary instead of the selected lot (PRD FR-12.1). */
  wholeView: boolean;
  /** The "more" panel of the selected lot is open (PRD FR-12.2). */
  moreOpen: boolean;
  select: (lotKey: string | null) => void;
  setWhole: (whole: boolean) => void;
  setMore: (open: boolean) => void;
}

export interface BikriDraftStores {
  header: UseBoundStore<StoreApi<HeaderStore>>;
  lots: EntryStore;
  /** The store of the sale lines of one lot. Made when first asked for. */
  salesFor: (lotKey: string) => EntryStore;
  /**
   * The sale lines of all lots as one list, for the "whole بکری" view (PRD FR-12.1). It holds no
   * data of its own: it shows the rows of the lots' stores and hands every change to the lot that
   * owns the row (union-store.ts).
   */
  all: EntryStore;
  /**
   * Asks the grid of the sale lines that is on screen to put the cursor on a cell: the line
   * `rowIndex` of the lot (or its last line), in the view of one lot or in the whole view.
   */
  focusSaleLine: (lotKey: string, rowIndex: number | 'last', field: string) => void;
  view: UseBoundStore<StoreApi<ViewState>>;
  /** Goes up by one on every change of anything typed. */
  version: UseBoundStore<StoreApi<{ value: number }>>;
}

/** A new, empty draft: today as both dates, one empty lot row. */
export function createBikriDraft(today: string): BikriDraftStores {
  const version = create<{ value: number }>()(() => ({ value: 0 }));
  const bump = () => {
    version.setState((state) => ({ value: state.value + 1 }));
  };
  const header = create<HeaderStore>()((set) => ({
    entryDate: today,
    billDate: '',
    book: '',
    truckNo: '',
    zamindar: null,
    adda: null,
    driverName: '',
    driverPhone: '',
    totalFreight: '',
    store: null,
    note: '',
    patch: (patch) => {
      set(patch);
    },
  }));
  header.subscribe(bump);

  const lots = createEntryStore();
  lots.subscribe(bump);

  const sales = new Map<string, EntryStore>();
  const salesFor = (lotKey: string): EntryStore => {
    const known = sales.get(lotKey);
    if (known !== undefined) return known;
    const made = createEntryStore();
    made.subscribe(bump);
    sales.set(lotKey, made);
    return made;
  };

  const view = create<ViewState>()((set) => ({
    selectedLot: lots.getState().rows[0]?.rowId ?? null,
    wholeView: false,
    moreOpen: false,
    select: (selectedLot) => {
      set({ selectedLot });
    },
    setWhole: (wholeView) => {
      set({ wholeView });
    },
    setMore: (moreOpen) => {
      set({ moreOpen });
    },
  }));

  const all = createLinesUnion({
    lotKeys: () => lots.getState().rows.map((row) => row.rowId),
    salesFor,
    select: (lotKey) => {
      view.getState().select(lotKey);
    },
    onChange: version.subscribe,
  });

  const focusSaleLine = (lotKey: string, rowIndex: number | 'last', field: string) => {
    if (!view.getState().wholeView) {
      salesFor(lotKey).getState().setPendingFocus({ rowIndex, field });
      return;
    }
    // In the whole view the line is found among the lines of all the lots before it.
    let before = 0;
    for (const lot of lots.getState().rows) {
      const count = salesFor(lot.rowId).getState().rows.length;
      if (lot.rowId === lotKey) {
        const at = rowIndex === 'last' ? count - 1 : rowIndex;
        all.getState().setPendingFocus({ rowIndex: before + at, field });
        return;
      }
      before += count;
    }
  };

  return { header, lots, salesFor, all, focusSaleLine, view, version };
}

const BikriDraftContext = createContext<BikriDraftStores | null>(null);
export const BikriDraftProvider = BikriDraftContext.Provider;

/** The draft of the screen. Used by the parts of the screen below the page. */
export function useBikriDraft(): BikriDraftStores {
  const stores = useContext(BikriDraftContext);
  if (stores === null) throw new Error('useBikriDraft is used outside the بکری screen.');
  return stores;
}
