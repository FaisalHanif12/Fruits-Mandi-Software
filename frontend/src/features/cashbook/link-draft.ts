// The link of a separate khata or of a person who is lent money, as the forms hold it (PRD FR-16.6,
// FR-16.13, plan 76 points 5 and 10): none (the firm's own), the shop (every movement is split
// between the partners by their shares of that day) or one partner (the whole of it is his). The
// backend splits and posts; this only holds what the user chose and builds the value it takes
// (CLAUDE.md rule 1).

import type { Link } from '../../bindings/Link';
import type { LinkKind } from '../../bindings/LinkKind';
import type { PersonType } from '../../bindings/PersonType';

/** The kinds a form can offer. A khata may have none; a person's loans are the shop's or a partner's. */
export type LinkChoiceKind = LinkKind;

/** What a link form holds: the kind, and the partner as the text of a Select (empty until chosen). */
export interface LinkDraft {
  kind: LinkChoiceKind;
  /** The partner's id as text, for a partner link. Empty when none is chosen. */
  partnerId: string;
}

/** A khata starts with no link: it is the firm's, as before. */
export const KHATA_LINK_KINDS: readonly LinkChoiceKind[] = ['none', 'shop', 'partner'];
/** A person's loans are linked to the shop or to one partner (PRD FR-16.13). */
export const PERSON_LINK_KINDS: readonly LinkChoiceKind[] = ['shop', 'partner'];

export const NO_LINK: LinkDraft = { kind: 'none', partnerId: '' };
export const SHOP_LINK: LinkDraft = { kind: 'shop', partnerId: '' };

/** The types a person can have, in the order of the list (PRD FR-16.4). */
export const PERSON_TYPES: readonly PersonType[] = [
  'staff',
  'garden_owner',
  'trip_person',
  'other',
];

/** The draft of a link the backend sent. */
export function linkDraftOf(link: Link): LinkDraft {
  return {
    kind: link.kind,
    partnerId: link.kind === 'partner' && link.partner_id !== null ? String(link.partner_id) : '',
  };
}

/**
 * The link the backend takes. A partner link with no partner chosen sends no partner, so the backend
 * answers `ERR_LINK_PARTNER_REQUIRED` on `link.partner_id` and the form shows it at its field.
 */
export function linkOf(draft: LinkDraft): Link {
  if (draft.kind !== 'partner') return { kind: draft.kind, partner_id: null };
  const id = Number(draft.partnerId);
  return {
    kind: 'partner',
    partner_id: draft.partnerId.trim() === '' || !Number.isInteger(id) ? null : id,
  };
}

/** The field paths a refusal of a link points at. */
export const LINK_FIELD_PATHS: readonly string[] = ['link', 'link.partner_id'];

/** A kind's Fluent id for its words. */
export function linkKindId(kind: LinkKind): string {
  return `cashbook-link-kind-${kind}`;
}

/** A kind's short word, for a list or a card (the long words of `linkKindId` are for the choice box). */
export function linkWordId(kind: LinkKind): string {
  return `cashbook-link-word-${kind}`;
}

/** A person type's Fluent id for its words. */
export function personTypeId(type: PersonType): string {
  return `cashbook-person-type-${type.replaceAll('_', '-')}`;
}
