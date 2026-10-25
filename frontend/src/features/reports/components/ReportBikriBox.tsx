// The box that picks the one بکری of the sale book report (R1, PRD 19.2, FR-12.13): the report is
// the lots of one بکری, so it cannot run without one. The user types the بکری number; the list is
// the backend's own find (`bikri_find`), and the first match is highlighted, so a number typed and
// Enter takes it. Like every search it takes the text as typed (Code Standards F41). Nothing is
// worked out or ordered here.

import { Localized } from '@fluent/react';
import type { BikriFindInput } from '../../../bindings/BikriFindInput';
import { isoToDisplayDate } from '../../../shared/format';
import { Code, Num, SuggestInput, type FieldControlProps } from '../../../shared/ui';
import { useBikriFind } from '../../bikri';
import type { ReportBikri } from '../filters';

interface ReportBikriBoxProps extends FieldControlProps {
  /** The بکری picked, or null. */
  bikri: ReportBikri | null;
  /** What is typed while none is picked. */
  text: string;
  onTextChange: (text: string) => void;
  onPick: (bikri: ReportBikri) => void;
}

/** A number typed as digits, for the find; anything else finds nothing. */
function numberOf(text: string): number | null {
  const trimmed = text.trim();
  return /^\d{1,9}$/.test(trimmed) ? Number(trimmed) : null;
}

export function ReportBikriBox({
  bikri,
  text,
  onTextChange,
  onPick,
  ...control
}: ReportBikriBoxProps) {
  const typed = bikri === null ? text : '';
  const find: BikriFindInput = {
    number: numberOf(typed),
    truck_no: null,
    seller_id: null,
    date: null,
    item_id: null,
  };
  const hits = useBikriFind(find, typed.trim() !== '').data ?? [];
  const rows = typed.trim() === '' ? [] : hits;
  // A بکری picked shows its number; a language switch changes nothing in it.
  const shown = bikri === null ? text : String(bikri.number);
  return (
    <SuggestInput
      {...control}
      value={shown}
      onChange={onTextChange}
      highlightFirst
      suggestions={rows.map((row) => ({
        value: String(row.number),
        content: (
          <span className="flex flex-wrap items-center gap-2">
            <Localized
              id="report-bikri-hit"
              vars={{ number: row.number, date: isoToDisplayDate(row.entry_date) }}
              elems={{ num: <Num /> }}
            >
              <span />
            </Localized>
            {row.truck_no === null || row.truck_no === '' ? null : <Code>{row.truck_no}</Code>}
          </span>
        ),
        onPick: () => {
          onPick({ id: row.id, number: row.number });
        },
      }))}
    />
  );
}
