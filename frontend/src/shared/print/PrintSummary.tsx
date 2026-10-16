// The amounts under the goods, how the buyer pays, and the amount in words. Kept in one piece:
// they never split across two pages. Amounts and the sentence in words come from the backend
// (PRD FR-19.7); this only draws them. A line is drawn by its role: a plain line, a subtotal with a
// rule above, the total with a double rule under it.

import { useLocalization } from '@fluent/react';
import { Num } from '../ui/Num';
import { PAYMENT_PAID_ID, type PrintAmount, type PrintPage } from './types';

const ROW_CLASS = {
  line: '',
  subtotal: 'print-amount-subtotal',
  total: 'print-amount-total',
} as const satisfies Record<PrintAmount['role'], string>;

export function PrintSummary({ page }: { page: PrintPage }) {
  const { l10n } = useLocalization();
  return (
    <div className="print-summary">
      {page.amounts.length === 0 ? null : (
        <table className="print-amounts">
          <tbody>
            {page.amounts.map((amount, index) => (
              <tr key={index} className={ROW_CLASS[amount.role]}>
                <th scope="row">{l10n.getString(amount.label_id)}</th>
                <td className="print-num">
                  <Num>{amount.amount}</Num>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      {page.payment_id === null ? null : (
        <p className="print-payment" data-paid={page.payment_id === PAYMENT_PAID_ID}>
          {l10n.getString(page.payment_id)}
        </p>
      )}
      {page.words === null ? null : (
        <p className="print-words">
          <span className="print-words-label">{l10n.getString(page.words.label_id)}</span>
          <span className="print-words-text">{page.words.text}</span>
        </p>
      )}
    </div>
  );
}
