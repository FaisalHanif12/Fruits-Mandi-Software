// The client's own layout of the buyer's bill (PRD R5 ادھاری تفصیل, FR-12.12, FR-19.12): ONE row for
// the buyer with the serial, the party (name with tag), the تفصیل with each line written
// "(item qty . rate)", the رقم (value of the goods) and the رقم بعد لاگا (value + buyer commission
// + لاگا, C5). The lines of تفصیل are written by the backend (`PrintLine.detail`) and shown as they
// are: the spaces round the dot stay, nothing wraps inside a line, and no left-to-right isolate is
// put round the whole text. The two amounts are the backend's own lines of the page, picked by their
// labels and never added up (CLAUDE.md rule 1).
//
// A line of an item sold by weight shows its weight in kilograms: its quantity is a نگ count with a
// rate per kilo, and "(کیلا 4 . 590)" would invite a wrong reading (plan 17e, Section 10 point 3,
// open question g of Umar).
//
// On the 80 mm roll there is no room for five columns: the same pieces are written one under the
// other.

import { useLocalization } from '@fluent/react';
import { Num } from '../ui/Num';
import { PrintName } from './PrintName';
import { R5_AMOUNT_IDS, type Paper, type PrintLine, type PrintPage } from './types';

interface PrintR5Props {
  page: PrintPage;
  paper: Paper;
}

/** The amount of the page with this label, or nothing when the page has none. */
function amountOf(page: PrintPage, labelId: string): string {
  return page.amounts.find((amount) => amount.label_id === labelId)?.amount ?? '';
}

function DetailLines({ lines }: { lines: readonly PrintLine[] }) {
  const { l10n } = useLocalization();
  return (
    <ul className="print-r5-details">
      {lines.map((line, index) => (
        <li key={index} className="print-r5-detail">
          {line.weight_kg === null
            ? line.detail
            : l10n.getString('print-r5-line-weight', {
                item: line.item,
                weight: line.weight_kg,
                rate: line.rate,
              })}
        </li>
      ))}
    </ul>
  );
}

export function PrintR5({ page, paper }: PrintR5Props) {
  const { l10n } = useLocalization();
  const value = amountOf(page, R5_AMOUNT_IDS.value);
  const total = amountOf(page, R5_AMOUNT_IDS.total);
  const party = <PrintName name={page.party.name} tag={page.party.tag} />;
  if (paper === 'mm80') {
    return (
      <div className="print-r5-compact">
        <div className="print-compact-row">
          <span className="print-compact-name">
            <Num>{page.serial}</Num> {party}
          </span>
        </div>
        <DetailLines lines={page.lines} />
        <div className="print-compact-row">
          <span>{l10n.getString('print-r5-col-value')}</span>
          <Num>{value}</Num>
        </div>
        <div className="print-compact-row print-compact-total">
          <span>{l10n.getString('print-r5-col-total')}</span>
          <Num>{total}</Num>
        </div>
      </div>
    );
  }
  return (
    <table className="print-table print-r5">
      <thead>
        <tr>
          <th scope="col" className="print-num print-col-qty">
            {l10n.getString('print-r5-col-serial')}
          </th>
          <th scope="col">{l10n.getString('print-r5-col-party')}</th>
          <th scope="col">{l10n.getString('print-r5-col-details')}</th>
          <th scope="col" className="print-num print-col-num">
            {l10n.getString('print-r5-col-value')}
          </th>
          <th scope="col" className="print-num print-col-num">
            {l10n.getString('print-r5-col-total')}
          </th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td className="print-num print-col-qty">
            <Num>{page.serial}</Num>
          </td>
          <td>{party}</td>
          <td>
            <DetailLines lines={page.lines} />
          </td>
          <td className="print-num print-col-num">
            <Num>{value}</Num>
          </td>
          <td className="print-num print-col-num">
            <Num>{total}</Num>
          </td>
        </tr>
      </tbody>
    </table>
  );
}
