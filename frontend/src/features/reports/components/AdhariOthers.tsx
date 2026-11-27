// The other buyers of ادھاری تفصیل, one row each (plan 97 item 6, PRD R5, Umar 24-11-2026): the buyers who owed at the start of
// the day and bought nothing today, by name with their tag and code, under a small heading that keeps the words of the print
// ("the other buyers who did not buy today"). The rows are the backend's, in the Urdu order of the name; the figures are the
// backend's and nothing is added up here (CLAUDE.md rule 1). Their total and the total credit of every buyer are the lines
// under the list (`TotalsLine` of the page), from the backend's totals.
//
// A long list (every buyer who owes can be hundreds) has a height of its own and scrolls inside, so the page stays short and
// only the rows in view are drawn (F34). Enter or a double click on a buyer opens his ledger (R13), as on the main list.

import { Localized } from '@fluent/react';
import { useId } from 'react';
import type { AdhariOtherRow } from '../../../bindings/AdhariOtherRow';
import { ListGrid, listHeightMode, isLongList } from '../../../shared/grid';
import { ADHARI_OTHER_COLUMNS } from '../adhari-columns';

interface AdhariOthersProps {
  rows: readonly AdhariOtherRow[];
  /** Opens the ledger of the buyer (R13). */
  onOpen: (row: AdhariOtherRow) => void;
}

/** The id of a row: the buyer's account, never his place in the list. */
function rowId(row: AdhariOtherRow): string {
  return String(row.party.account_id);
}

export function AdhariOthers({ rows, onOpen }: AdhariOthersProps) {
  const headingId = useId();
  if (rows.length === 0) return null;
  return (
    <section aria-labelledby={headingId} className="flex flex-col gap-2">
      <Localized id="print-adhari-sheet-others">
        <h2 id={headingId} className="type-label text-ink" />
      </Localized>
      <div className={isLongList(rows.length) ? 'h-tree-view' : undefined}>
        <ListGrid<AdhariOtherRow>
          columns={ADHARI_OTHER_COLUMNS}
          rows={rows}
          getRowId={rowId}
          labelId="print-adhari-sheet-others"
          onOpen={onOpen}
          heightMode={listHeightMode(rows.length)}
        />
      </div>
    </section>
  );
}
