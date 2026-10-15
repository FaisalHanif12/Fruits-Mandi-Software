// Types of the vouchers. What the backend owns comes from the generated bindings (Code Standards
// F2). What the user types is text (CLAUDE.md rule 1): the form holds an `AdvanceDraft`, and
// amounts the backend sends are `Paisa` and `Balance`, shown and never worked out.

import type { AdvancePreviewRow } from '../../bindings/AdvancePreviewRow';
import type { AdvancesInfo } from '../../bindings/AdvancesInfo';
import type { AdvancesPreview } from '../../bindings/AdvancesPreview';
import type { OpenAdvance } from '../../bindings/OpenAdvance';
import type { VoucherMode } from '../../bindings/VoucherMode';
import type { VoucherSaved } from '../../bindings/VoucherSaved';
import type { VoucherType } from '../../bindings/VoucherType';
import type { PickerValue } from '../../shared/ui';

export type {
  AdvancePreviewRow,
  AdvancesInfo,
  AdvancesPreview,
  OpenAdvance,
  VoucherMode,
  VoucherSaved,
  VoucherType,
};

/** One voucher as the form holds it, all as text (CLAUDE.md rule 1). */
export interface AdvanceDraft {
  /** `DD-MM-YYYY` as typed. */
  date: string;
  seller: PickerValue | null;
  /** Whole rupees as typed. A simple sum such as `5000+2000` is read by the backend (FR-7.48). */
  amount: string;
  mode: VoucherMode;
  bank: PickerValue | null;
  cheque_no: string;
  /** `DD-MM-YYYY` as typed. */
  cheque_date: string;
  expected_goods: string;
  narration: string;
}

/** A draft that has a seller: the one thing the backend takes as an id and not as text. */
export type ReadyAdvance = AdvanceDraft & { seller: NonNullable<AdvanceDraft['seller']> };
