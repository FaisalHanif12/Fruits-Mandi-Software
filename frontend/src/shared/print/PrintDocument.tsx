// A whole printed document: every sheet of one print job, in the print language. The templates
// of the modules (features/<module>/print/) choose the columns; everything else is here, so every
// print of the app looks like one family (Code Standards F12). The root (language, direction, paper,
// digit style and footer) is `PrintRoot`, shared with every other print.

import { useLocalization } from '@fluent/react';
import type { Ref } from 'react';
import { PrintRoot } from './PrintRoot';
import { PrintSheet } from './PrintSheet';
import { LAYOUT_IDS, type PrintColumn, type PrintData, type PrintPage } from './types';

export interface PrintDocumentProps {
  data: PrintData;
  /** The goods columns of each page. */
  columnsOf: (page: PrintPage, data: PrintData) => readonly PrintColumn[];
  /**
   * How the goods of a sheet are laid out: a table of lines (default), or the client's R5 columns,
   * one row for the buyer (serial, party with tag, تفصیل, رقم, رقم بعد لاگا; PRD R5, FR-12.12).
   */
  layout?: 'goods' | 'r5';
  /** True in the hidden print window (see PrintRoot). */
  applyRoot?: boolean;
  ref?: Ref<HTMLDivElement>;
}

function Sheets({
  data,
  columnsOf,
  layout,
}: Pick<PrintDocumentProps, 'data' | 'columnsOf' | 'layout'>) {
  const { l10n } = useLocalization();
  if (data.pages.length === 0) {
    return <p className="print-empty">{l10n.getString(LAYOUT_IDS.empty)}</p>;
  }
  return (
    <>
      {data.pages.map((page, index) => (
        <PrintSheet
          key={index}
          data={data}
          page={page}
          columns={columnsOf(page, data)}
          {...(layout === undefined ? {} : { layout })}
        />
      ))}
    </>
  );
}

export function PrintDocument({
  data,
  columnsOf,
  layout,
  applyRoot = false,
  ref,
}: PrintDocumentProps) {
  return (
    <PrintRoot
      frame={data}
      paper={data.paper}
      applyRoot={applyRoot}
      {...(ref === undefined ? {} : { ref })}
    >
      <Sheets data={data} columnsOf={columnsOf} {...(layout === undefined ? {} : { layout })} />
    </PrintRoot>
  );
}
