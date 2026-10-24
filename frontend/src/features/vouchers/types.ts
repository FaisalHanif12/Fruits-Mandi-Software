// Types of the vouchers. What the backend owns comes from the generated bindings (Code Standards
// F2). What the user types is text (CLAUDE.md rule 1): the form holds a `VoucherDraft`, and
// amounts the backend sends are `Paisa` and `Balance`, shown and never worked out.

import type { AdvanceEffect } from '../../bindings/AdvanceEffect';
import type { AdvanceInput } from '../../bindings/AdvanceInput';
import type { AdvancesInfo } from '../../bindings/AdvancesInfo';
import type { BillEffect } from '../../bindings/BillEffect';
import type { CollectionBuyer } from '../../bindings/CollectionBuyer';
import type { CollectionGroup } from '../../bindings/CollectionGroup';
import type { CollectionList } from '../../bindings/CollectionList';
import type { LineSource } from '../../bindings/LineSource';
import type { OpenAdvance } from '../../bindings/OpenAdvance';
import type { OpenBill } from '../../bindings/OpenBill';
import type { OpenBills } from '../../bindings/OpenBills';
import type { OpenPayables } from '../../bindings/OpenPayables';
import type { PayablePosting } from '../../bindings/PayablePosting';
import type { PaymentInput } from '../../bindings/PaymentInput';
import type { ReceiptInput } from '../../bindings/ReceiptInput';
import type { ReceiptRowResult } from '../../bindings/ReceiptRowResult';
import type { ReceiptsSaved } from '../../bindings/ReceiptsSaved';
import type { SavedPukhta } from '../../bindings/SavedPukhta';
import type { VoucherCancelled } from '../../bindings/VoucherCancelled';
import type { VoucherEntry } from '../../bindings/VoucherEntry';
import type { VoucherPreview } from '../../bindings/VoucherPreview';
import type { VoucherRow } from '../../bindings/VoucherRow';
import type { VoucherSaved } from '../../bindings/VoucherSaved';
import type { VoucherType } from '../../bindings/VoucherType';
import type { VouchersList } from '../../bindings/VouchersList';
import type { AccountType } from '../../shared/accounts';
import type { PickerValue } from '../../shared/ui';
import type { ScreenType } from './screen-type';

export type {
  AdvanceEffect,
  AdvanceInput,
  AdvancesInfo,
  BillEffect,
  CollectionBuyer,
  CollectionGroup,
  CollectionList,
  LineSource,
  OpenAdvance,
  OpenBill,
  OpenBills,
  OpenPayables,
  PayablePosting,
  PaymentInput,
  ReceiptInput,
  ReceiptRowResult,
  ReceiptsSaved,
  SavedPukhta,
  VoucherCancelled,
  VoucherEntry,
  VoucherPreview,
  VoucherRow,
  VoucherSaved,
  VoucherType,
  VouchersList,
};

/** Who a وصولی comes from: a buyer (the usual case), or a seller returning advance money (FR-15.12). */
export type FromSide = 'buyer' | 'seller';

/** One voucher as the form holds it, all as text (CLAUDE.md rule 1). */
export interface VoucherDraft {
  type: ScreenType;
  /** `DD-MM-YYYY` as typed. */
  date: string;
  /** The account chosen, as the picker holds it. */
  account: PickerValue | null;
  /** What kind of account it is, so the screen knows which helpers and fields to show. */
  accountType: AccountType | null;
  /** Receipt only. */
  from: FromSide;
  /** The one-time person, for a پختہ کھاتہ account (PRD FR-13.5); null for any other account. */
  person: SavedPukhta | null;
  /** The text in the person field: what the user typed to search, or the name he picked. */
  personText: string;
  /** Whole rupees as typed. A simple sum such as `5000+2000` is read by the backend (FR-7.48). */
  amount: string;
  /** Receipt only: the collector (اگراہی والا), a staff member. */
  collector: PickerValue | null;
  /** Payment to a buyer only: why the cash is lent (PRD FR-15.11). */
  reason: string;
  /** Advance only: expected goods or season (PRD 15.4). */
  expected_goods: string;
  narration: string;
}
