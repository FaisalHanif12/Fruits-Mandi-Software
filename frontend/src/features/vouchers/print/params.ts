// The address of the hidden print window for a voucher: `#/print?voucher=<id>&doc=receipt_slip
// &language=<ur|en>&job=<job>` (plan 15 contract 9). The window is opened by the backend, so every
// value is read as plain text and checked here. A value that is not right is a failed print, never
// a guess. The job name is checked by the same function as the sale book's (one rule for the window).

import type { Language } from '../../../bindings/Language';
import type { VoucherPrintDataInput } from '../../../bindings/VoucherPrintDataInput';
import { jobOf, positiveInt } from '../../../shared/print';

export interface VoucherPrintParams {
  /** The job name; the events are `print-ready-<job>` and `print-error-<job>`. */
  job: string;
  input: VoucherPrintDataInput;
}

const LANGUAGE_VALUES: readonly Language[] = ['ur', 'en'];
/** True when the address is for a voucher (it names one), even if the rest is wrong. */
export function isVoucherAddress(searchStr: string): boolean {
  return new URLSearchParams(searchStr).has('voucher');
}

/** The parameters of the print window for a voucher, or null when the address is not valid. */
export function parseVoucherPrintParams(searchStr: string): VoucherPrintParams | null {
  const search = new URLSearchParams(searchStr);
  const job = jobOf(searchStr);
  const voucherId = positiveInt(search.get('voucher'));
  const language = LANGUAGE_VALUES.find((candidate) => candidate === search.get('language'));
  if (job === null) return null;
  if (voucherId === null || search.get('doc') !== 'receipt_slip' || language === undefined) {
    return null;
  }
  return { job, input: { voucher_id: voucherId, language } };
}
