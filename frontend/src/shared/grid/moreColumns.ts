// "More columns" for a list (PRD FR-7.2, NFR-USE-15, Code Standards F22). A list never scrolls
// sideways, so a column that does not fit beside the others at a layout step waits behind a tick.
// With the tick on, the added columns come at every step, and a few less needed columns step aside
// only at the steps where the room is short. Every column of the PRD is then reachable on screen.
//
// Example (the list of بکری, PRD FR-12.7): the نگ column is added after the items. At the wide step
// the day and time give up their place, because 1,310 px of columns plus 140 px would not fit the
// 1,332 px that are left beside the side menu.

import type { LayoutStep } from '../hooks/useLayoutStep';
import type { ListColumn } from './ListGrid';

export interface MoreColumnsPlan<T> {
  /** The columns the tick adds. Each one is put after the base column named in `after`. */
  readonly added: readonly { readonly column: ListColumn<T>; readonly after: string }[];
  /** The columns that step aside while the tick is on, at the steps listed (and no other). */
  readonly makesRoom: Readonly<Partial<Record<LayoutStep, readonly string[]>>>;
}

/**
 * The columns of a list for the layout step and the "More columns" tick. Without the tick they are
 * the base columns, each shown from the step it asks for. With it, the added columns are shown at
 * every step and the ones in `makesRoom` for this step are left out.
 */
export function columnsWithMore<T>(
  base: readonly ListColumn<T>[],
  plan: MoreColumnsPlan<T>,
  step: LayoutStep,
  more: boolean,
): readonly ListColumn<T>[] {
  if (!more) return base;
  const out: ListColumn<T>[] = [];
  for (const column of base) {
    out.push(column);
    for (const item of plan.added) {
      if (item.after === column.field) out.push({ ...item.column, from: 'compact' });
    }
  }
  const aside = new Set(plan.makesRoom[step] ?? []);
  return out.filter((column) => !aside.has(column.field));
}
