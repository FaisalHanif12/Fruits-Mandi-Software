// One sheet: one seller, one buyer or one one-time buyer (plan 14 contract 5). The header, the
// facts, who it is for, the goods, the amounts, how the buyer pays and the amount in words. Each
// sheet starts on a new page when printed (print.css). It lays out what it is given and works
// out nothing.

import { useLocalization } from '@fluent/react';
import { PrintGoods } from './PrintGoods';
import { PrintHeader } from './PrintHeader';
import { PrintPartyBlock } from './PrintPartyBlock';
import { PrintR5 } from './PrintR5';
import { PrintScreenFooter } from './PrintScreenFooter';
import { PrintSummary } from './PrintSummary';
import { LAYOUT_IDS, type PrintColumn, type PrintData, type PrintPage } from './types';

interface PrintSheetProps {
  data: PrintData;
  page: PrintPage;
  /** The goods columns of this page. Empty when the document has no goods (a receipt slip). */
  columns: readonly PrintColumn[];
  /** The goods as a table of lines, or in the client's R5 columns (see PrintDocument). */
  layout?: 'goods' | 'r5';
}

export function PrintSheet({ data, page, columns, layout = 'goods' }: PrintSheetProps) {
  const { l10n } = useLocalization();
  return (
    <section className="print-sheet">
      {data.cancelled ? (
        <div className="print-stamp" aria-hidden="true">
          <span className="print-stamp-text">{l10n.getString(LAYOUT_IDS.cancelled)}</span>
        </div>
      ) : null}
      <PrintHeader header={data.header} />
      {layout === 'r5' ? (
        <PrintR5 page={page} paper={data.paper} />
      ) : (
        <>
          <PrintPartyBlock party={page.party} />
          {page.lines.length === 0 || columns.length === 0 ? null : (
            <PrintGoods lines={page.lines} columns={columns} paper={data.paper} />
          )}
        </>
      )}
      <PrintSummary page={page} />
      {/* On paper the footer is the page margin (print.css). This line is for the preview. */}
      <PrintScreenFooter footer={data.footer} />
    </section>
  );
}
