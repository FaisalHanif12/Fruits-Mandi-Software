// The party card (PRD FR-7.38, NFR-USE-27, Code Standards F40): everything about a party, shown
// next to an account field as soon as the account is known. Name with father or firm, city,
// phone, current balance with its side as a word, last entry date; for a buyer his credit limit
// and how much of it is used; for a seller his open advance. Warnings come with a word and an icon
// as well as a colour (NFR-USE-12). Every figure is read from the books by the backend; nothing is
// worked out here, and a card is never reused from an earlier look (balances change with every
// posting).

import { Localized } from '@fluent/react';
import { CircleAlert, TriangleAlert } from 'lucide-react';
import { usePartyCard, type CardWarning, type PartyCardData } from '../accounts';
import { uiErrorOf } from '../api';
import { isoToDisplayDate } from '../format';
import { displayName, useLanguage } from '../i18n';
import { Balance } from './Balance';
import { Code } from './Code';
import { ErrorMessage } from './ErrorMessage';
import { Num } from './Num';

interface PartyCardProps {
  /** The account to show, or null for nothing. */
  accountId: number | null;
}

/** An inactive account is a warning (amber); the others are about money (red). */
const WARNING_STYLE: Record<CardWarning, string> = {
  inactive: 'border-warn-border bg-warn-soft text-warn',
  over_credit_limit: 'border-due bg-due-soft text-due',
  owes_firm: 'border-due bg-due-soft text-due',
};

const WARNING_ID: Record<CardWarning, string> = {
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
        <span className="type-section text-ink">
          {displayName({ name_ur: card.name_ur, name_en: card.name_en }, language)}
        </span>
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
        {card.credit_limit_paisa === null ? null : (
          <Fact labelId="assist-card-credit-limit">
            <Num paisa={card.credit_limit_paisa} />
          </Fact>
        )}
        {card.credit_used_paisa === null ? null : (
          <Fact labelId="assist-card-credit-used">
            <Num paisa={card.credit_used_paisa} />
          </Fact>
        )}
        {card.advance_open_paisa === null ? null : (
          <Fact labelId="assist-card-advance">
            <Num paisa={card.advance_open_paisa} />
          </Fact>
        )}
      </div>
      {card.warnings.length === 0 ? null : (
        <ul className="flex flex-wrap gap-2">
          {card.warnings.map((warning) => (
            <li
              key={warning}
              className={`flex items-center gap-2 rounded-1 border px-3 type-body ${WARNING_STYLE[warning]}`}
            >
              {warning === 'inactive' ? (
                <TriangleAlert aria-hidden="true" className="size-icon-sm shrink-0" />
              ) : (
                <CircleAlert aria-hidden="true" className="size-icon-sm shrink-0" />
              )}
              <Localized id={WARNING_ID[warning]} />
            </li>
          ))}
        </ul>
      )}
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
