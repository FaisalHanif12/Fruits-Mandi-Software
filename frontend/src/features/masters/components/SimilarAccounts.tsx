// Accounts that look like the one being typed, below its name, before save (PRD FR-7.63, FR-10.7):
// code, name, father or firm, city, phone and balance, in either script, so two people with the
// same name are easy to tell apart. The tag (نشان) shows after the name through displayName, and a
// help line says that the same name with another tag is not a duplicate (PRD 10.2, FR-10.7, E38).
// The backend marks the account whose name, tag and phone all match (`is_duplicate`): its row says so
// in words, and the title of the panel changes. The frontend never decides this itself. It only
// warns; the save is never blocked by it.

import { Localized } from '@fluent/react';
import { TriangleAlert } from 'lucide-react';
import { useLanguage, displayName } from '../../../shared/i18n';
import { Code } from '../../../shared/ui';
import type { AccountMatch } from '../../../shared/accounts';

interface SimilarAccountsProps {
  matches: readonly AccountMatch[];
}

export function SimilarAccounts({ matches }: SimilarAccountsProps) {
  const language = useLanguage();
  if (matches.length === 0) return null;
  return (
    <section className="flex flex-col gap-2 rounded-2 border border-warn-border bg-warn-soft p-3">
      <h3 className="flex items-center gap-2 type-body font-semibold text-warn">
        <TriangleAlert aria-hidden="true" className="size-icon-md shrink-0" />
        <Localized
          id={
            matches.some((match) => match.is_duplicate)
              ? 'accounts-similar-duplicate-title'
              : 'accounts-similar-title'
          }
        />
      </h3>
      <ul className="flex flex-col gap-1">
        {matches.map((match) => (
          <li
            key={match.id}
            className="flex flex-wrap items-baseline gap-x-4 gap-y-1 type-body text-ink"
          >
            <Code>{match.code}</Code>
            <span className="font-semibold">{displayName(match, language)}</span>
            <span className="text-ink-muted">
              {displayName(
                { name_ur: match.father_or_firm_ur, name_en: match.father_or_firm_en },
                language,
              )}
            </span>
            <span className="text-ink-muted">
              {displayName({ name_ur: match.city_ur, name_en: match.city_en }, language)}
            </span>
            {match.phone1 === '' ? null : <Code>{match.phone1}</Code>}
            {match.is_duplicate ? (
              <Localized id="accounts-similar-duplicate-chip">
                <span className="rounded-1 border border-due bg-due-soft px-2 type-chip text-due" />
              </Localized>
            ) : null}
          </li>
        ))}
      </ul>
      <Localized id="accounts-similar-tag-help">
        <p className="type-body text-ink-muted" />
      </Localized>
    </section>
  );
}
