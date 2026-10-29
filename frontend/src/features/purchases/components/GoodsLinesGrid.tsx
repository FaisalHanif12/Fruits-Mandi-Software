// The grid of item lines of one entry (PRD FR-12.41): a trip purchase entry, or a garden's goods
// arrived entry. Each line holds an item, a packing, the quantity (and the weight of an item sold
// by weight), for a purchase the rate and the costs, and the split between our shop and another
// shop (PRD FR-12.41): the quantity for another shop and the shop's name, with what is left for
// our shop shown by the backend. One save writes every line.
//
// - The grid is the shared one (shared/grid): Enter, the arrow keys, Ctrl+Delete and the pickers
//   come from it (PRD 7.2, Code Standards F27). This file only declares the columns and puts the
//   words around the grid. Enter stops only at the item, the packing, the quantity and the rate
//   (and the weight of a weight item); the other cells are reached by click or arrows (plan 35, D3).
// - The lines live in the store above the grid, so a language switch rebuilds the grid and nothing
//   typed is lost (F39).
// - What the backend refused on a line is shown on its cell and the words are the cell's tooltip;
//   the list under the form tells it too, so colour is never the only signal (NFR-USE-12).
// - Nothing here works out money or a quantity: the value of a line and the quantity left for our
//   shop are the preview's (`calc`).

import { Localized } from '@fluent/react';
import { Plus } from 'lucide-react';
import { useMemo } from 'react';
import { EntryGrid, type EntryStore } from '../../../shared/grid';
import type { Working } from '../../../shared/ui';
import { Button } from '../../../shared/ui';
import { useLayoutStep } from '../../../shared/hooks/useLayoutStep';
import { goodsLineColumns, LINE, type GoodsKind } from '../lines';

type PerRow<T> = Readonly<Record<string, Readonly<Record<string, T>>>>;

interface GoodsLinesGridProps {
  kind: GoodsKind;
  /** The rows of the lines, kept above the grid. */
  store: EntryStore;
  /** The preview's words per line: the value, and the quantity left for our shop. */
  calc?: PerRow<string> | undefined;
  /** What the backend refused, per line and cell, as words. */
  cellErrors?: PerRow<string> | undefined;
  /** How the value of each line was worked out, behind its ⓘ (PRD FR-7.43). */
  workings?: PerRow<Working> | undefined;
  /**
   * Which rows hold an item sold by weight, read when Enter is pressed: Enter stops at the weight of
   * those rows only (plan 35, PRD FR-12.41). It must stay the same function between renders.
   */
  weightRows?: (() => ReadonlySet<string>) | undefined;
}

/** Puts the cursor on the item of the line just added. */
export function addGoodsLine(store: EntryStore): void {
  store.getState().addRow();
  store.getState().setPendingFocus({ rowIndex: 'last', field: LINE.item });
}

export function GoodsLinesGrid({
  kind,
  store,
  calc,
  cellErrors,
  workings,
  weightRows,
}: GoodsLinesGridProps) {
  const step = useLayoutStep();
  const columns = useMemo(() => goodsLineColumns(kind, step, weightRows), [kind, step, weightRows]);
  return (
    <section aria-labelledby={`goods-lines-heading-${kind}`} className="flex flex-col gap-2">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <Localized id="goods-lines-title">
          <h3 id={`goods-lines-heading-${kind}`} className="type-section text-ink" />
        </Localized>
        <Button
          onClick={() => {
            addGoodsLine(store);
          }}
        >
          <Plus aria-hidden="true" className="size-icon-md" />
          <Localized id="goods-lines-add" />
        </Button>
      </div>
      <Localized
        id={kind === 'purchase' ? 'goods-lines-help-purchase' : 'goods-lines-help-arrival'}
      >
        <p className="type-body text-ink-muted" />
      </Localized>
      <EntryGrid
        columns={columns}
        store={store}
        labelId={kind === 'purchase' ? 'goods-lines-label-purchase' : 'goods-lines-label-arrival'}
        {...(calc === undefined ? {} : { calc })}
        {...(cellErrors === undefined ? {} : { cellErrors })}
        {...(workings === undefined ? {} : { workings })}
        fit
        heightMode="auto"
      />
    </section>
  );
}
