// Where a bill or a payable posting came from, in words (PRD FR-7.58, FR-7.59): a بکری with its
// number, an opening balance, cash lent, a جنرل entry. The number is the backend's and is isolated
// (F25). The backend sends the kind; the words are Fluent's.

import { Localized } from '@fluent/react';
import type { LineSource } from '../types';

const SOURCE_ID: Record<LineSource, string> = {
  bikri: 'vouchers-source-bikri',
  opening: 'vouchers-source-opening',
  cash_lent: 'vouchers-source-cash-lent',
  receipt: 'vouchers-source-receipt',
  general: 'vouchers-source-general',
  purchase: 'vouchers-source-purchase',
  fruit: 'vouchers-source-fruit',
  other: 'vouchers-source-other',
};

export function SourceLabel({ source, number }: { source: LineSource; number: number | null }) {
  return (
    <Localized id={SOURCE_ID[source]} vars={{ number: number === null ? '' : String(number) }}>
      <span />
    </Localized>
  );
}
