// The columns of the page behind the dashboard tile of garden owners and others (PRD FR-16.13, plan 76 point 10):
// the entries of the month (money given and returned, with the person, type and link of each) and the people with
// their open amounts. Every figure is the backend's; the lists add up nothing (CLAUDE.md rule 1). A partner's link
// is shown by his name, through `displayName` (F38). Money and dates are centred (NFR-USE-19).

import type { LentDuesPerson } from '../../bindings/LentDuesPerson';
import type { LentDuesRow } from '../../bindings/LentDuesRow';
import type { Link } from '../../bindings/Link';
import { isoToDisplayDate } from '../../shared/format';
import type { ListColumn, ListContext } from '../../shared/grid';
import { displayName } from '../../shared/i18n';
import { linkWordId, personTypeId } from './link-draft';

/** A partner's two names by his id, from the partner list the page reads. */
export type PartnerNames = ReadonlyMap<number, { name_ur: string; name_en: string }>;

/** The link in words: "The shop", "Nobody" or the partner's name. */
export function linkText(link: Link, names: PartnerNames, context: ListContext): string {
  if (link.kind !== 'partner' || link.partner_id === null) {
    return context.text(linkWordId(link.kind));
  }
  const partner = names.get(link.partner_id);
  return partner === undefined
    ? context.text(linkWordId('partner'))
    : displayName(partner, context.language);
}

/** The columns of the entries of the month. */
export function entryColumns(names: PartnerNames): readonly ListColumn<LentDuesRow>[] {
  return [
    {
      field: 'date',
      type: 'date',
      headerId: 'cashbook-col-date',
      value: (row) => isoToDisplayDate(row.entry_date),
    },
    {
      field: 'person',
      type: 'name',
      headerId: 'cashbook-dues-col-person',
      grow: true,
      value: (row, { language }) => displayName(row.person, language),
    },
    {
      field: 'type',
      type: 'text',
      headerId: 'cashbook-dues-col-type',
      from: 'normal',
      value: (row, { text }) => text(personTypeId(row.person_type)),
    },
    {
      field: 'link',
      type: 'text',
      headerId: 'cashbook-dues-col-link',
      from: 'normal',
      value: (row, context) => linkText(row.link, names, context),
    },
    {
      field: 'kind',
      type: 'text',
      headerId: 'cashbook-dues-col-kind',
      value: (row, { text }) =>
        text(row.kind === 'given' ? 'cashbook-dues-kind-given' : 'cashbook-dues-kind-returned'),
    },
    {
      field: 'reason',
      type: 'text',
      headerId: 'cashbook-col-reason',
      grow: true,
      wrap: true,
      from: 'normal',
      value: (row) => row.reason,
    },
    {
      field: 'amount',
      type: 'money',
      headerId: 'cashbook-dues-col-amount',
      value: (row, { money }) => money(row.amount_paisa),
    },
  ];
}

/** The columns of the people with something open. */
export function peopleColumns(names: PartnerNames): readonly ListColumn<LentDuesPerson>[] {
  return [
    {
      field: 'person',
      type: 'name',
      headerId: 'cashbook-dues-col-person',
      grow: true,
      value: (row, { language }) => displayName(row.person, language),
    },
    {
      field: 'type',
      type: 'text',
      headerId: 'cashbook-dues-col-type',
      from: 'normal',
      value: (row, { text }) => text(personTypeId(row.person_type)),
    },
    {
      field: 'link',
      type: 'text',
      headerId: 'cashbook-dues-col-link',
      from: 'normal',
      value: (row, context) => linkText(row.link, names, context),
    },
    {
      field: 'lent',
      type: 'money',
      headerId: 'cashbook-dues-col-lent',
      from: 'wide',
      value: (row, { money }) => money(row.lent_paisa),
    },
    {
      field: 'returned',
      type: 'money',
      headerId: 'cashbook-dues-col-returned',
      from: 'wide',
      value: (row, { money }) => money(row.returned_paisa),
    },
    {
      field: 'open',
      type: 'money',
      headerId: 'cashbook-dues-col-open',
      value: (row, { money }) => money(row.open_paisa),
    },
  ];
}
