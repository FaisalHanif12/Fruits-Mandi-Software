// The page behind the dashboard tile "Garden owners' and others' dues" (route /others-dues; PRD FR-9.1, FR-16.13,
// FR-19.11, plan 76 point 10). It reads ادھار رقم: every person who is not staff (a garden owner, a market-trip
// person, anyone else), with what was given and what came back, one month at a time. At the top: what is
// still open and how many people it is, and what was given and returned in the month. Then the people with
// something open, and the entries of the month, each with its date, person, type, reason and link. The
// filters are the person type and the link (the shop, or one partner). All figures, the counts and the
// totals are the backend's, over the whole filter; the page adds nothing up (CLAUDE.md rule 1).
//
// - One month at a time, opening on the current month and never going past it nor before the first month
//   with an entry: the buttons, the go-to-month box, Page Up and Page Down (as the other month lists).
// - ↑ ↓ choose a row; Enter opens the person on the ادھار رقم screen, to give or take back money.
// - Four states: loading, an error with a retry, nothing in this month (the buttons still work), and the rows.

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { ArrowLeft, HandCoins } from 'lucide-react';
import { useMemo, useState } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import type { LentDuesPerson } from '../../../bindings/LentDuesPerson';
import type { LentDuesRow } from '../../../bindings/LentDuesRow';
import type { LinkKind } from '../../../bindings/LinkKind';
import type { PersonType } from '../../../bindings/PersonType';
import type { RecordRef } from '../../../bindings/RecordRef';
import { uiErrorOf } from '../../../shared/api';
import { PagedList } from '../../../shared/grid';
import { useListWindow } from '../../../shared/hooks';
import { displayName, useLanguage } from '../../../shared/i18n';
import { SHORTCUTS, formNavigationProps, useEscape } from '../../../shared/keyboard';
import {
  Button,
  Card,
  EmptyState,
  ErrorState,
  Fact,
  Field,
  ListPage,
  ListWindowBar,
  LoadingState,
  Num,
  PartnerLockedBox,
  Select,
} from '../../../shared/ui';
import { usePartnerList } from '../../partners';
import { useLentDues, useLentDuesPeople, type LentDuesFilter } from '../hooks';
import { entryColumns, peopleColumns, type PartnerNames } from '../dues-columns';
import { PERSON_TYPES, personTypeId } from '../link-draft';

/** The type filter: every type but staff (what the tile counts), or one type. */
type TypeFilter = 'default' | PersonType;
type LinkFilter = 'all' | Exclude<LinkKind, 'none'>;

const LINK_FILTERS: readonly LinkFilter[] = ['all', 'shop', 'partner'];

/** The person as the lent screen takes him in its address: a whole picker value. */
function pickerOf(person: RecordRef) {
  return {
    id: person.id,
    code: person.code,
    name_ur: person.name_ur,
    name_en: person.name_en,
    ...(person.tag === undefined || person.tag === '' ? {} : { tag: person.tag }),
  };
}

function entryId(row: LentDuesRow): string {
  return `${row.kind}-${String(row.voucher_id ?? `opening-${String(row.person.id)}`)}-${row.entry_date}`;
}

export function OthersDuesPage() {
  const navigate = useNavigate();
  const language = useLanguage();
  const { l10n } = useLocalization();
  const day = useDayStatus();
  const partners = usePartnerList();
  const [type, setType] = useState<TypeFilter>('default');
  const [link, setLink] = useState<LinkFilter>('all');
  const [partnerId, setPartnerId] = useState('');
  const [focusId, setFocusId] = useState<string | null>(null);
  const listWindow = useListWindow({
    kind: 'month',
    source: { kind: 'lent_dues' },
    today: day.data?.today,
  });
  useEscape('leave', () => {
    void navigate({ to: '/' });
  });

  const names: PartnerNames = useMemo(
    () => new Map((partners.data ?? []).map((partner) => [partner.id, partner])),
    [partners.data],
  );
  const entries = useMemo(() => entryColumns(names), [names]);
  const people = useMemo(() => peopleColumns(names), [names]);

  // While the partner section is locked the backend names no partner (a partner link has no partner) and refuses a
  // filter on one (`ERR_PARTNER_LOCKED` at `partner_id`), so none is sent then (PRD FR-17.11, FR-16.13). It is known
  // from the answer, or from that refusal when the section was locked after the page was opened.
  const [refusedPartner, setRefusedPartner] = useState(false);
  const filter: LentDuesFilter = useMemo(
    () => ({
      person_types: type === 'default' ? [] : [type],
      link_kinds: link === 'all' ? [] : [link],
      partner_id:
        !refusedPartner && link === 'partner' && partnerId !== '' ? Number(partnerId) : null,
      month: listWindow.month,
    }),
    [type, link, refusedPartner, partnerId, listWindow.month],
  );
  const list = useLentDues(filter, listWindow.ready);
  // The people have pages of their own: the backend says how many there are in all.
  const peopleList = useLentDuesPeople(filter, listWindow.ready);
  const first = list.data?.pages[0];
  const answerLocked = first?.partner_locked === true;
  const locked = answerLocked || refusedPartner;
  const refused =
    list.isError && uiErrorOf(list.error).errors.some((item) => item.code === 'ERR_PARTNER_LOCKED');
  if (refused && !refusedPartner) setRefusedPartner(true);
  // Unlocked again: a fresh answer says so (not one kept from before, which may be out of date).
  if (
    refusedPartner &&
    first !== undefined &&
    !answerLocked &&
    !list.isFetching &&
    !list.isPlaceholderData
  ) {
    setRefusedPartner(false);
  }
  // A refusal shows as the state, not as the rows of the filter before it.
  const rows = list.isError ? undefined : list.data?.pages.flatMap((page) => page.rows);
  const peopleRows = peopleList.isError
    ? undefined
    : peopleList.data?.pages.flatMap((page) => page.people);

  const openPerson = (person: RecordRef) => {
    void navigate({ to: '/rokar/lent', search: { type: 'money_lent', party: pickerOf(person) } });
  };

  const typeOptions: readonly { value: TypeFilter; label: string }[] = [
    { value: 'default', label: l10n.getString('cashbook-dues-type-default') },
    ...PERSON_TYPES.map((candidate) => ({
      value: candidate,
      label: l10n.getString(personTypeId(candidate)),
    })),
  ];

  const filters = (
    <div {...formNavigationProps} className="flex flex-col gap-3">
      <div className="grid grid-cols-2 gap-3 normal:grid-cols-3">
        <Field label={<Localized id="cashbook-dues-filter-type" />}>
          {(props) => (
            <Select<TypeFilter>
              {...props}
              value={type}
              onChange={setType}
              options={typeOptions.map((option) => ({ value: option.value, label: option.label }))}
            />
          )}
        </Field>
        <Field label={<Localized id="cashbook-dues-filter-link" />}>
          {(props) => (
            <Select<LinkFilter>
              {...props}
              value={link}
              onChange={(next) => {
                setLink(next);
                if (next !== 'partner') setPartnerId('');
              }}
              options={LINK_FILTERS.map((candidate) => ({
                value: candidate,
                label: <Localized id={`cashbook-dues-link-${candidate}`} />,
              }))}
            />
          )}
        </Field>
        {link === 'partner' && locked ? (
          // The partner's name is not shown without the code, so he cannot be chosen: the box stands there.
          <div className="col-span-2 normal:col-span-1">
            <PartnerLockedBox />
          </div>
        ) : link === 'partner' ? (
          <Field label={<Localized id="cashbook-link-partner" />}>
            {(props) => (
              <Select
                {...props}
                readOnly={props.readOnly === true || partners.isPending}
                value={partnerId === '' ? undefined : partnerId}
                onChange={setPartnerId}
                options={(partners.data ?? [])
                  .filter((partner) => partner.is_active)
                  .map((partner) => ({
                    value: String(partner.id),
                    label: displayName(partner, language),
                  }))}
              />
            )}
          </Field>
        ) : null}
      </div>
    </div>
  );

  let body;
  if (list.isError && first === undefined) {
    body = (
      <ErrorState
        title={<Localized id="cashbook-dues-failed" />}
        error={uiErrorOf(list.error)}
        onRetry={() => {
          void list.refetch();
        }}
      />
    );
  } else if (first === undefined || rows === undefined) {
    body = <LoadingState rows={6} />;
  } else {
    body = (
      <div className="flex flex-col gap-3">
        {/* The amounts and totals are the shop's and show; only the partner of a link is hidden. */}
        {locked && link !== 'partner' ? <PartnerLockedBox /> : null}
        <dl className="grid grid-cols-4 gap-3">
          <Fact stat label={<Localized id="cashbook-dues-open" />}>
            <Num inherit paisa={first.totals.open_paisa} />
          </Fact>
          <Fact stat label={<Localized id="cashbook-dues-people" />}>
            <Num inherit>{String(first.totals.open_people_count)}</Num>
          </Fact>
          <Fact stat label={<Localized id="cashbook-dues-given" />}>
            <Num inherit paisa={first.totals.given_paisa} />
          </Fact>
          <Fact stat label={<Localized id="cashbook-dues-returned" />}>
            <Num inherit paisa={first.totals.returned_paisa} />
          </Fact>
        </dl>
        <Card compact icon={HandCoins} title={<Localized id="cashbook-dues-people-title" />}>
          <PagedList<LentDuesPerson>
            columns={people}
            rows={peopleRows}
            getRowId={(row) => String(row.person.id)}
            labelId="cashbook-dues-people-label"
            failedId="cashbook-dues-failed"
            error={peopleList.isError ? uiErrorOf(peopleList.error) : null}
            onRetry={() => {
              void peopleList.refetch();
            }}
            pager={peopleList.pager}
            onOpen={(row) => {
              openPerson(row.person);
            }}
            autoHeight
            empty={
              <EmptyState
                icon={HandCoins}
                title={<Localized id="cashbook-dues-people-empty" />}
                help={<Localized id="cashbook-dues-people-empty-help" />}
              />
            }
          />
        </Card>
        <Card compact icon={HandCoins} title={<Localized id="cashbook-dues-entries-title" />}>
          <PagedList<LentDuesRow>
            columns={entries}
            rows={rows}
            getRowId={entryId}
            labelId="cashbook-dues-entries-label"
            failedId="cashbook-dues-failed"
            error={list.isError ? uiErrorOf(list.error) : null}
            onRetry={() => {
              void list.refetch();
            }}
            pager={list.pager}
            onPageKey={listWindow.onPageKey}
            onOpen={(row) => {
              openPerson(row.person);
            }}
            onFocusRow={setFocusId}
            focusRowId={focusId}
            autoHeight
            empty={
              <EmptyState
                icon={HandCoins}
                title={<Localized id="cashbook-dues-empty" />}
                help={<Localized id="cashbook-dues-empty-help" />}
              />
            }
          />
        </Card>
        <ListWindowBar window={listWindow} />
      </div>
    );
  }

  return (
    <ListPage
      icon={HandCoins}
      titleId="cashbook-dues-title"
      help={
        <Localized id="cashbook-dues-help">
          <p />
        </Localized>
      }
      actions={
        <Button
          shortcut={SHORTCUTS.back}
          onClick={() => {
            void navigate({ to: '/' });
          }}
        >
          <ArrowLeft aria-hidden="true" className="size-icon-md rtl:rotate-180" />
          <Localized id="cashbook-dues-back" />
        </Button>
      }
      filters={filters}
    >
      {body}
    </ListPage>
  );
}
