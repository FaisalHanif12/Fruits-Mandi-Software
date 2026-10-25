// Account and chart types. Plan 07 types come from the generated bindings (Code Standards F2);
// the names below are the names the screens already use. (Plan 08:
// short codes, the party card, global search) comes from the plan 08 bindings too.
// Money from the backend is `Paisa` (shown, never worked out); money typed by the user is text.

import type { AccountFilter } from '../../bindings/AccountFilter';
import type { AccountHit as AccountHitBinding } from '../../bindings/AccountHit';
import type { AccountScope } from '../../bindings/AccountScope';
import type { CardWarning } from '../../bindings/CardWarning';
import type { CardWarningKind } from '../../bindings/CardWarningKind';
import type { BikriHit as BikriHitBinding } from '../../bindings/BikriHit';
import type { VoucherHit as VoucherHitBinding } from '../../bindings/VoucherHit';
import type { PartyCard } from '../../bindings/PartyCard';
import type { ResolveStatus } from '../../bindings/ResolveStatus';
import type { AccountNode } from '../../bindings/AccountNode';
import type { AccountRow } from '../../bindings/AccountRow';
import type { AccountsList } from '../../bindings/AccountsList';
import type { AccountType } from '../../bindings/AccountType';
import type { Balance } from '../../bindings/Balance';
import type { BalanceSide } from '../../bindings/BalanceSide';
import type { CoaControl } from '../../bindings/CoaControl';
import type { CoaHead } from '../../bindings/CoaHead';
import type { ControlNode } from '../../bindings/ControlNode';
import type { HeadNode } from '../../bindings/HeadNode';
import type { RecordRef } from '../../bindings/RecordRef';
import type { SimilarAccount } from '../../bindings/SimilarAccount';

export type { AccountFilter, AccountRow, AccountType, Balance, BalanceSide };
export type { OpeningLineInput } from '../../bindings/OpeningLineInput';
export type { OpeningPreviewRow } from '../../bindings/OpeningPreviewRow';
export type { OpeningsInput } from '../../bindings/OpeningsInput';
export type { OpeningsPreview } from '../../bindings/OpeningsPreview';
export type { OpeningsSaved } from '../../bindings/OpeningsSaved';

/** A record another one points at, for a picker to show. */
export type AccountRef = RecordRef;
/** The list with its count and footer totals. */
export type AccountList = AccountsList;
/** A close match shown while a name is typed (PRD FR-7.63, FR-10.7). */
export type AccountMatch = SimilarAccount;
/** The chart in three levels (PRD FR-10.1): head, control, account. */
export type ChartHead = HeadNode;
export type ChartControl = ControlNode;
export type ChartAccount = AccountNode;
/** The plain lists of heads and controls (the control list says where "other" may go). */
export type HeadItem = CoaHead;
export type ControlItem = CoaControl;

/** What the account form holds, all as text (CLAUDE.md rule 1). */
export interface AccountDraft {
  id: number | null;
  version: number | null;
  account_type: AccountType;
  control_id: number | null;
  code: string;
  names: { urdu: string; english: string };
  /**
   * The tag after the name (PRD 10.2): empty, or 1 to 4 English capital letters and dots. Typed as
   * text, never turned into Urdu; the backend makes it capital and checks it (`ERR_TAG_INVALID`).
   */
  tag: string;
  father: { urdu: string; english: string };
  city: AccountRef | null;
  adda: AccountRef | null;
  phone1: string;
  phone2: string;
  address: string;
  linked_account: AccountRef | null;
  /** `DD-MM-YYYY` as typed. */
  opening_date: string;
  opening_dr: string;
  opening_cr: string;
  opening_is_advance: boolean;
  credit_allowed: boolean;
  credit_limit: string;
  default_marka: AccountRef | null;
  collector: AccountRef | null;
  /** 0 (Monday) to 6 (Sunday), or null for no day. */
  collection_weekday: number | null;
  is_active: boolean;
  note: string;
}

// ---- Plan 08: short codes, the party card, search.

export type { AccountScope, CardWarning, CardWarningKind, ResolveStatus };

/**
 * One account in a picker or a search (PRD FR-7.40). The balance comes from a second call
 * (`accounts_balances`), so the search itself stays fast; null when it is not known (the row then
 * shows no balance, never a wrong one).
 */
export interface AccountHit extends AccountHitBinding {
  balance: Balance | null;
}

/**
 * What a code typed in an account field means (PRD FR-7.39, FR-7.42), asked on Enter or on leaving
 * the field, never while typing: `1` may be the start of `12`.
 */
export interface ResolvedCode {
  status: ResolveStatus;
  account: AccountHit | null;
  candidates: AccountHit[];
  /** For an unknown code: the full code a new account would get. */
  suggested_code: string | null;
}

/** The party card (PRD FR-7.38): everything about a party, read from the books. */
export type PartyCardData = PartyCard;

/** An account found by the global search (Ctrl+K). */
export interface AccountSearchHit {
  kind: 'account';
  id: number;
  code: string;
  name_ur: string;
  name_en: string;
  /** The tag (نشان) after the name, shown by `displayName`. Absent when there is none. */
  tag?: string;
  extra_ur: string;
  extra_en: string;
}

/**
 * A بکری found by the global search, by its number or by the start of its truck number (PRD
 * FR-5.4). Opening it opens the بکری screen on this id (plan 13).
 */
export interface BikriSearchHit {
  kind: 'bikri';
  id: number;
  number: number;
  /** `YYYY-MM-DD`. */
  entry_date: string;
  truck_no: string | null;
  status: BikriHitBinding['status'];
  matched: BikriHitBinding['matched'];
}

/**
 * A voucher found by the global search, by its number (PRD FR-5.4, plan 15): the digits find the
 * voucher with that number in each series, `R-12` or `و-12` finds one series. Opening it opens the
 * voucher screen on this id.
 */
export interface VoucherSearchHit {
  kind: 'voucher';
  id: number;
  voucher_type: VoucherHitBinding['voucher_type'];
  number: number;
  /** `YYYY-MM-DD`. */
  entry_date: string;
  /** The account of the voucher (both names), or null. */
  account: VoucherHitBinding['account'];
  amount_paisa: VoucherHitBinding['amount_paisa'];
  status: string;
}

/** One result of the global search (Ctrl+K): an account, a بکری or a voucher. */
export type GlobalHit = AccountSearchHit | BikriSearchHit | VoucherSearchHit;
