// Which reports show one window at a time (PRD FR-19.11, plan 76 point 8): the registers with many rows a day
// (the sales registers, the day book, the cash book, the voucher registers) show 7 days; the lists with few rows a
// day (the expense report, the fruit register, a partner's statement) show one month. The window opens on the
// latest one and is the range the report is asked for, unless the Owner types dates of his own in the
// boxes: those win. The aggregates, the balances as of a day and the party ledgers are not windowed (they
// are one table of the whole filter or have their own windows). Pure rules; there is no money here.

import type { ReportId } from '../../bindings/ReportId';
import type { WindowSource } from '../../bindings/WindowSource';
import type { ListWindowKind } from '../../shared/hooks';
import type { ReportFilterState } from './filters';

export interface ReportWindow {
  kind: ListWindowKind;
  source: WindowSource;
}

/** The window of a report, or null when it has none. `state` gives the partner of R31. */
export function reportWindowOf(id: ReportId, state: ReportFilterState): ReportWindow | null {
  switch (id) {
    case 'R2':
    case 'R3':
    case 'R4':
    case 'R6':
    case 'R27':
      return { kind: 'days', source: { kind: 'sales' } };
    case 'R11':
      return { kind: 'days', source: { kind: 'journal' } };
    case 'R12':
      return { kind: 'days', source: { kind: 'cashbook' } };
    case 'R15':
      return { kind: 'days', source: { kind: 'vouchers', voucher_type: null } };
    case 'R23':
      return { kind: 'days', source: { kind: 'vouchers', voucher_type: 'receipt' } };
    case 'R25':
      return { kind: 'days', source: { kind: 'vouchers', voucher_type: 'payment' } };
    case 'R29':
      return { kind: 'month', source: { kind: 'expenses' } };
    case 'R37':
      return { kind: 'month', source: { kind: 'fruit' } };
    case 'R31':
      // A partner's statement needs the partner: until he is chosen there is nothing to window.
      return state.partner === null
        ? null
        : { kind: 'month', source: { kind: 'partner_statement', partner_id: state.partner } };
    default:
      return null;
  }
}

/** True when the Owner typed dates of his own (or chose a shortcut): they win over the window. */
export function hasOwnDates(state: ReportFilterState): boolean {
  return state.period !== 'custom' || state.from.trim() !== '' || state.to.trim() !== '';
}
