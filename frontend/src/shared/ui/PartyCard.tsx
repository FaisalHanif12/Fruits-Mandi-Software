// The party card (PRD FR-7.38, NFR-USE-27, Code Standards F40): everything about a party, shown
// next to an account field as soon as the account is known. Name with father or firm, city,
// phone, current balance with its side as a word, last entry date; for a buyer his credit limit
// and how much of it is used; for a seller his open advance. Warnings come with a word and an icon
// as well as a colour (NFR-USE-12). Every figure is read from the books by the backend; nothing is
// worked out here, and a card is never reused from an earlier look (balances change with every
// posting).

import { Localized, useLocalization } from '@fluent/react';
import { CircleAlert, TriangleAlert } from 'lucide-react';
import { usePartyCard, type CardWarningKind, type PartyCardData } from '../accounts';
import { uiErrorOf } from '../api';
import { isoToDisplayDate } from '../format';
import { displayName, useLanguage } from '../i18n';
import { Balance } from './Balance';
import { Code } from './Code';
import { useDeadAmountActions } from './dead-amount';
import { DeadInfoLine } from './DeadInfoLine';
import { ErrorMessage } from './ErrorMessage';
import { LinkButton } from './LinkButton';
import { Num } from './Num';
import { usePartyCardLinks } from './party-card-links';

interface PartyCardProps {
  /** The account to show, or null for nothing. */
  accountId: number | null;
}

/** An inactive account is a warning (amber); the others are about money (red). */
const WARNING_STYLE: Record<CardWarningKind, string> = {
  inactive: 'border-warn-border bg-warn-soft text-warn',
  over_credit_limit: 'border-due bg-due-soft text-due',
  owes_firm: 'border-due bg-due-soft text-due',
};

const WARNING_ID: Record<CardWarningKind, string> = {
  inactive: 'assist-warning-inactive',
  over_credit_limit: 'assist-warning-over-credit-limit',
  owes_firm: 'assist-warning-owes-firm',
};

function Fact({ labelId, children }: { labelId: string; children: React.ReactNode }) {
  return (
    <span className="flex items-baseline gap-2">
      <Localized id={labelId}>
        <span className="text-ink-muted" />
      </Localized>
      {children}
    </span>
  );
}

/**
 * The ledger and the statement of this account, one press away (PRD FR-7.65). The range is filled
 * by the report that opens: the whole history, from the first day of data. Shown only where the app
 * frame can open a report.
 */
function ReportButtons({ accountId }: { accountId: number }) {
  const open = usePartyCardLinks();
  const { l10n } = useLocalization();
  if (open === null) return null;
  return (
    <div
      role="group"
      aria-label={l10n.getString('report-card-reports-label')}
      className="flex flex-wrap items-center gap-x-5"
    >
      <LinkButton
        onClick={() => {
          open('R13', accountId);
        }}
      >
        <Localized id="report-card-ledger" />
      </LinkButton>
      <LinkButton
        onClick={() => {
          open('R14', accountId);
        }}
      >
        <Localized id="report-card-statement" />
      </LinkButton>
    </div>
  );
}

/**
 * The two actions of the dead amount (PRD FR-10.10): move a person who owes the shop and is gone,
 * or make him active again. A person can be moved when he is active, is a seller, a buyer or
 * someone with money lent, and his balance is بنام; the backend still decides when the dialog asks
 * (`masters_dead_preview`). Shown only where the app frame can open the dialogs.
 */
function DeadButtons({ card }: { card: PartyCardData }) {
  const actions = useDeadAmountActions();
  if (actions === null) return null;
  const names = { name_ur: card.name_ur, name_en: card.name_en, tag: card.tag ?? null };
  if (card.dead !== null) {
    const markId = card.dead.mark_id;
    return (
      <div className="flex flex-wrap items-center gap-x-5">
        <LinkButton
          onClick={() => {
            actions.undo({ markId, names });
          }}
        >
          <Localized id="dead-undo-button" />
        </LinkButton>
      </div>
    );
  }
  const movable = card.is_active && card.account_type !== 'other' && card.balance.side === 'dr';
  if (!movable) return null;
  return (
    <div className="flex flex-wrap items-center gap-x-5">
      <LinkButton
        onClick={() => {
          actions.move({ accountId: card.id, personKey: null, names });
        }}
      >
        <Localized id="dead-move-button" />
      </LinkButton>
    </div>
  );
}

function CardBody({ card }: { card: PartyCardData }) {
  const language = useLanguage();
  const father = displayName(
    { name_ur: card.father_or_firm_ur, name_en: card.father_or_firm_en },
    language,
  );
  const city = displayName({ name_ur: card.city_ur, name_en: card.city_en }, language);
  return (
    <>
      <div className="flex flex-wrap items-baseline gap-x-3 gap-y-1">
        <span className="type-section text-ink">{displayName(card, language)}</span>
        <Code>{card.code}</Code>
        <Localized id={`accounts-type-${card.account_type}`}>
          <span className="type-chip rounded-1 bg-surface-sunken px-2 text-ink-muted" />
        </Localized>
      </div>
      <p className="flex flex-wrap items-baseline gap-x-4 gap-y-1 type-body text-ink-muted">
        {father === '' ? null : <span>{father}</span>}
        {city === '' ? null : <span>{city}</span>}
        {card.phone === '' ? null : <Code>{card.phone}</Code>}
      </p>
      <div className="flex flex-wrap items-baseline gap-x-6 gap-y-1 type-body text-ink">
        <Fact labelId="assist-card-balance">
          <Balance balance={card.balance} />
        </Fact>
        <Fact labelId="assist-card-last-entry">
          {card.last_entry_date === null ? (
            <Localized id="assist-card-never" />
          ) : (
            <Num>{isoToDisplayDate(card.last_entry_date)}</Num>
          )}
        </Fact>
        {card.credit_limit === null ? null : (
          <Fact labelId="assist-card-credit-limit">
            <Num paisa={card.credit_limit} />
          </Fact>
        )}
        {card.credit_used === null ? null : (
          <Fact labelId="assist-card-credit-used">
            <Num paisa={card.credit_used} />
          </Fact>
        )}
        {card.advance_open === null ? null : (
          <Fact labelId="assist-card-advance">
            <Num paisa={card.advance_open} />
          </Fact>
        )}
      </div>
      {card.warnings.length === 0 ? null : (
        <ul className="flex flex-wrap gap-2">
          {card.warnings.map((warning) => (
            <li
              key={warning.kind}
              className={`flex items-center gap-2 rounded-1 border px-3 type-body ${WARNING_STYLE[warning.kind]}`}
            >
              {warning.kind === 'inactive' ? (
                <TriangleAlert aria-hidden="true" className="size-icon-sm shrink-0" />
              ) : (
                <CircleAlert aria-hidden="true" className="size-icon-sm shrink-0" />
              )}
              <Localized id={WARNING_ID[warning.kind]} />
              {warning.amount_paisa === null ? null : <Num paisa={warning.amount_paisa} />}
            </li>
          ))}
        </ul>
      )}
      {card.dead === null ? null : <DeadInfoLine info={card.dead} />}
      <ReportButtons accountId={card.id} />
      <DeadButtons card={card} />
    </>
  );
}

export function PartyCard({ accountId }: PartyCardProps) {
  const query = usePartyCard(accountId);
  if (accountId === null) return null;
  return (
    <section
      aria-live="polite"
      aria-busy={query.isPending}
      className="flex flex-col gap-2 rounded-2 border border-border bg-surface p-3"
    >
      {query.isPending ? (
        <Localized id="assist-card-loading">
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : query.isError ? (
        <div role="alert" className="flex flex-col gap-1 text-due">
          <Localized id="assist-card-failed">
            <p className="type-body" />
          </Localized>
          {uiErrorOf(query.error).errors.map((item, index) => (
            <ErrorMessage key={`${item.code}-${String(index)}`} item={item}>
              <p className="type-body" />
            </ErrorMessage>
          ))}
        </div>
      ) : (
        <CardBody card={query.data} />
      )}
    </section>
  );
}
