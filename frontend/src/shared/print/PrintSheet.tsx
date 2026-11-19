// One sheet: one seller, one buyer or one one-time buyer (plan 14 contract 5). The header, the
// facts, who it is for, the goods, the amounts, how the buyer pays and the amount in words. Each
// sheet starts on a new page when printed (print.css). It lays out what it is given and works
// out nothing.

import { useLocalization } from '@fluent/react';
import { PrintGoods } from './PrintGoods';
import { NAMED_TITLE_IDS, PrintHeader } from './PrintHeader';
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

/** The title that names the party ("<name> کا بل"), or nothing when the document's title is not one (plan 78 A3). */
function namedTitle(
  getString: (id: string, vars?: Record<string, string>) => string,
  titleId: string,
  page: PrintPage,
): string | undefined {
  if (!NAMED_TITLE_IDS.has(titleId)) return undefined;
  const { name, tag } = page.party;
  return getString(titleId, { name: tag === undefined ? name : `${name} ${tag}` });
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
      <PrintHeader
        header={data.header}
        title={namedTitle((id, vars) => l10n.getString(id, vars), data.header.title_id, page)}
      />
      {/* A reprint after a correction cannot be taken for the first print (PRD FR-7.71). */}
      {data.revised_on === null ? null : (
        <p className="print-revised">
          {l10n.getString('edit-print-revised', { date: data.revised_on })}
        </p>
      )}
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
