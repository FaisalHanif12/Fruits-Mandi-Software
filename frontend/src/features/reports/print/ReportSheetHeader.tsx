// The top of one printed sheet of a report (PRD FR-19.13): the firm (name, mandi, contacts), the title
// of the print, its filters (dates and party, FR-19.10) and the print time. A report and every slip of
// the credit buyers (R5, one sheet for each buyer) start with this, so the two look the same. The
// backend wrote every word and figure in the print language; this lays them out.

import { useLocalization } from '@fluent/react';
import type { PrintFact } from '../../../bindings/PrintFact';
import type { PrintFirm } from '../../../bindings/PrintFirm';
import { Num } from '../../../shared/ui/Num';

interface ReportSheetHeaderProps {
  firm: PrintFirm;
  titleId: string;
  filters: readonly PrintFact[];
  printedAt: string;
  /** True on the party day sheet: it is for one day, so its box says "Day", not "As of". */
  dayFilter?: boolean;
}

export function ReportSheetHeader({
  firm,
  titleId,
  filters,
  printedAt,
  dayFilter = false,
}: ReportSheetHeaderProps) {
  const { l10n } = useLocalization();
  const contact = [firm.address, firm.phone].filter((part) => part.trim() !== '');
  return (
    <>
      <header className="print-header">
        <div className="print-firm">
          <p className="print-firm-name">{firm.name}</p>
          {firm.mandi === '' ? null : <p className="print-firm-line">{firm.mandi}</p>}
          {contact.length === 0 ? null : (
            <p className="print-firm-line">
              {contact.map((part, index) => (
                <span key={part}>
                  {index > 0 ? ' · ' : null}
                  <bdi dir="auto">{part}</bdi>
                </span>
              ))}
            </p>
          )}
        </div>
        <h1 className="print-title">{l10n.getString(titleId)}</h1>
      </header>
      <dl className="print-facts">
        {filters.map((fact) => (
          <div key={fact.label_id} className="print-fact">
            <dt>
              {l10n.getString(
                dayFilter && fact.label_id === 'report-filter-as-of'
                  ? 'report-filter-day'
                  : fact.label_id,
              )}
            </dt>
            <dd>{fact.value === '' ? null : <bdi dir="auto">{fact.value}</bdi>}</dd>
          </div>
        ))}
        <div className="print-fact">
          <dt>{l10n.getString('print-printed-at')}</dt>
          <dd>
            <Num>{printedAt}</Num>
          </dd>
        </div>
      </dl>
    </>
  );
}
