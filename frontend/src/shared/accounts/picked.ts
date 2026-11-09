// The one way a record chosen in a picker is kept on a screen (PRD 10.2, Code Standards F38): its
// id and code, both names, and its tag (نشان) when it has one. Every screen that keeps a chosen
// account, from a hit of the picker or from a saved record, builds it here, so the tag can never
// be lost on the way and a language switch still shows the name with its tag.
//
// A چکتا buyer (PRD Section 13) is kept the same way: his `id` and `code` are those of the account
// چکتا کھاتہ (0030), which is what a sale line posts to, his names are his own, and `chukta` holds
// what else the line must carry: his phone, and his own record id when the picker knew it.
import type { ChuktaBuyer } from '../../bindings/ChuktaBuyer';
import type { AccountHit } from './types';

/** The code of the account چکتا کھاتہ (PRD FR-13.1): every چکتا buyer's line is on it. */
const CHUKTA_KHATA_CODE = '0030';

/** What a picker hit, a record reference or an account row says about a record. */
export interface PickedSource {
  readonly id: number;
  readonly code: string;
  readonly name_ur: string;
  readonly name_en: string;
  readonly tag?: string | null | undefined;
  /** True for an item sold by weight, as the search hit of an item says (PRD FR-12.41, D3). */
  readonly sold_by_weight?: boolean | undefined;
  /** `chukta` for a چکتا buyer's hit of the buyer picker (PRD FR-7.29); anything else is an account. */
  readonly kind?: string | undefined;
  /** His own record, for a `chukta` hit (the party card reads it). */
  readonly chukta_id?: number | null | undefined;
  /** His phone, for a `chukta` hit that has one. */
  readonly phone?: string | null | undefined;
  /** What an already kept چکتا buyer says (a record that went through `pickedOf` once). */
  readonly chukta?: PickedChukta | undefined;
}

/**
 * The part of a چکتا buyer a sale line carries besides his names (PRD FR-13.2, FR-13.5): his
 * phone ("" if none), and his record id, or null when the line was read back from a saved one (the
 * saved line keeps his names and phone, not his record).
 */
export interface PickedChukta {
  readonly id: number | null;
  readonly phone: string;
}

/**
 * The record as a screen keeps it. `tag` is left out when there is none, which is also the shape
 * of a `RecordRef` from the backend, so it can go wherever one is expected. `soldByWeight` is there
 * only for an item that is sold by weight, so the shape of every other record stays the same.
 */
export interface PickedAccount {
  readonly id: number;
  readonly code: string;
  readonly name_ur: string;
  readonly name_en: string;
  readonly tag?: string;
  readonly soldByWeight?: true;
  /** Present only for a چکتا buyer (PRD FR-7.29). */
  readonly chukta?: PickedChukta;
}

/**
 * A چکتا buyer a party card is asked for (PRD FR-13.6): his names and phone, and his own record id
 * when the screen has it. With the id the card is the backend's; without it (a saved line read back
 * has his names and phone but not his record) the card is drawn from what the line holds.
 */
export interface CardChukta {
  readonly id: number | null;
  readonly name_ur: string;
  readonly name_en: string;
  readonly phone: string;
}

/** The card target of a chosen record: his names and phone, or null for an ordinary account. */
export function cardChuktaOf(
  picked:
    | {
        readonly name_ur: string;
        readonly name_en: string;
        readonly chukta?: PickedChukta | undefined;
      }
    | null
    | undefined,
): CardChukta | null {
  if (picked?.chukta === undefined) return null;
  return {
    id: picked.chukta.id,
    name_ur: picked.name_ur,
    name_en: picked.name_en,
    phone: picked.chukta.phone,
  };
}

/** Is this chosen record a چکتا buyer? (PRD FR-13.5: his line goes to the account چکتا کھاتہ.) */
export function isChuktaPick(
  picked: { readonly chukta?: PickedChukta | undefined } | null | undefined,
): boolean {
  return picked?.chukta !== undefined;
}

/** What is needed to tell two chosen records apart: the id, and for a چکتا buyer who he is. */
interface PickIdentity {
  readonly id: number;
  readonly name_ur: string;
  readonly name_en: string;
  readonly chukta?: PickedChukta | undefined;
}

/**
 * Are these two chosen records the same? Ordinary records are the same when their ids are. Every
 * چکتا buyer shares the id of the account چکتا کھاتہ (PRD FR-13.1), so one of them is told by his
 * names and phone (FR-13.2): changing Shahid to Nasir on a line is a change, though the id stays.
 */
export function samePick(
  first: PickIdentity | null | undefined,
  second: PickIdentity | null | undefined,
): boolean {
  if (first === null || first === undefined || second === null || second === undefined) {
    return (first ?? null) === (second ?? null);
  }
  if (first.id !== second.id) return false;
  if (first.chukta === undefined || second.chukta === undefined) {
    return first.chukta === second.chukta;
  }
  return (
    first.name_ur === second.name_ur &&
    first.name_en === second.name_en &&
    first.chukta.phone === second.chukta.phone
  );
}

/** What a picker row says about itself that tells a چکتا buyer from an account (PRD FR-7.29). */
interface HitIdentity {
  readonly id: number;
  readonly name_ur: string;
  readonly name_en: string;
  readonly kind?: string | undefined;
  readonly chukta_id?: number | null | undefined;
  readonly phone?: string | null | undefined;
}

/** The key of a picker row. Every چکتا hit has the id of the account چکتا کھاتہ, so his own record tells them apart. */
export function hitKey(hit: HitIdentity): string {
  return hit.kind === 'chukta' ? `chukta-${String(hit.chukta_id ?? hit.id)}` : String(hit.id);
}

/**
 * Is this row of the picker the record the field already holds? An account is the same when the
 * ids are; a چکتا buyer when his own record is, or, for one read back from a saved line (no record
 * id kept), when his names and phone are (PRD FR-13.2).
 */
export function isHitValue(hit: HitIdentity, value: PickIdentity): boolean {
  if (hit.id !== value.id) return false;
  if (hit.kind !== 'chukta') return value.chukta === undefined;
  if (value.chukta === undefined) return false;
  if (value.chukta.id !== null) return value.chukta.id === hit.chukta_id;
  return (
    value.name_ur === hit.name_ur &&
    value.name_en === hit.name_en &&
    value.chukta.phone === (hit.phone ?? '')
  );
}

export function pickedOf(source: PickedSource): PickedAccount {
  const chukta: PickedChukta | undefined =
    source.kind === 'chukta'
      ? { id: source.chukta_id ?? null, phone: source.phone ?? '' }
      : source.chukta;
  const base = {
    id: source.id,
    code: source.code,
    name_ur: source.name_ur,
    name_en: source.name_en,
    ...(source.sold_by_weight === true ? { soldByWeight: true as const } : {}),
    ...(chukta === undefined ? {} : { chukta }),
  };
  return source.tag === undefined || source.tag === null || source.tag === ''
    ? base
    : { ...base, tag: source.tag };
}

/**
 * The picker row of a چکتا buyer who was just added with the second add row (PRD FR-7.69, FR-13.3),
 * as the buyer box of a بکری line lists him: the account چکتا کھاتہ's id, which the backend sends
 * with him, and his own record in `chukta_id`. The box takes this row, so the line posts to the
 * account and carries his names and phone (PRD FR-13.5). His phone is the second line of the row, and
 * he has no balance. `control_id` is not read for a چکتا row (only the Index forms use a control).
 */
export function chuktaHitOf(buyer: ChuktaBuyer): AccountHit {
  return {
    id: buyer.account_id,
    code: CHUKTA_KHATA_CODE,
    name_ur: buyer.name_ur,
    name_en: buyer.name_en,
    extra_ur: buyer.phone ?? '',
    extra_en: buyer.phone ?? '',
    account_type: 'buyer',
    kind: 'chukta',
    chukta_id: buyer.id,
    phone: buyer.phone,
    control_id: 0,
    is_active: true,
    is_dead: false,
    balance: null,
  };
}
