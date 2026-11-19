// The records offered after "کون سا؟" or when a name is not found (PRD FR-7.77): a button for each,
// with the name and its tag, the code, the city and the balance with its side, so two people with
// one name are easy to tell apart. A lot (plan 71 L9) shows its name, "Ehsan, Mango-2", the day it came
// and the نگ received, so two lots of one seller and one item are easy to tell apart. A click goes on with the same question and counts nothing more.
// The balance is the backend's cell, with ✓ when its two places agree. At the end of a list of lots the
// backend adds one more button with the key `l*`: "all lots" of that seller and item. Its words are ours
// (Fluent, in both languages), never the name the backend sends with it.

import { Localized, useLocalization } from '@fluent/react';
import { Check } from 'lucide-react';
import type { AnswerOption } from '../../../bindings/AnswerOption';
import { isoToDisplayDate } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { Code, Num } from '../../../shared/ui';
import { useListContext } from '../../analytics';
import { NameWithCode } from './AnswerText';
import { figureParts } from './Figure';

/** The key of the button "all lots" at the end of a list of lots; a pick with it means every lot. */
export const ALL_LOTS_KEY = 'l*';

interface OptionsListProps {
  options: readonly AnswerOption[];
  onPick: (key: string) => void;
}

export function OptionsList({ options, onPick }: OptionsListProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const context = useListContext();
  return (
    <ul className="flex list-none flex-wrap gap-2">
      {options.map((option) => {
        if (option.key === ALL_LOTS_KEY) {
          return (
            <li key={option.key}>
              <button
                type="button"
                onClick={() => {
                  onPick(option.key);
                }}
                className="flex min-h-control items-center rounded-2 border border-border-control bg-surface px-4 py-2 text-start text-ink transition-calm"
              >
                <span className="type-label">
                  <Localized id="assistant-option-all-lots" />
                </span>
              </button>
            </li>
          );
        }
        const balance = option.balance === null ? null : figureParts(option.balance, context);
        return (
          <li key={option.key}>
            <button
              type="button"
              onClick={() => {
                onPick(option.key);
              }}
              className="flex min-h-control flex-col items-start gap-1 rounded-2 border border-border-control bg-surface px-4 py-2 text-start text-ink transition-calm"
            >
              <span className="type-label">
                <NameWithCode name={option.name} />
                {option.dead ? <span> {context.text('dead-tag')}</span> : null}
              </span>
              <span className="flex flex-wrap items-center gap-x-3 type-body text-ink-muted">
                {option.city === null ? null : (
                  <span>
                    {displayName({ name_ur: option.city.ur, name_en: option.city.en }, language)}
                  </span>
                )}
                {option.date === undefined ? null : (
                  <Localized
                    id="assistant-option-lot-day"
                    vars={{ date: isoToDisplayDate(option.date) }}
                    elems={{ date: <Code /> }}
                  >
                    <span />
                  </Localized>
                )}
                {option.qty === undefined ? null : (
                  <Localized
                    id="assistant-option-lot-qty"
                    vars={{ qty: String(option.qty) }}
                    elems={{ num: <Num /> }}
                  >
                    <span />
                  </Localized>
                )}
                {balance === null ? null : (
                  <span className="inline-flex items-center gap-1">
                    {l10n.getString('assistant-option-balance-label')}
                    <Num>{balance.value}</Num>
                    {balance.side === '' ? null : <span>{balance.side}</span>}
                    {option.checked ? (
                      <span className="inline-flex items-center text-money-in">
                        <Check aria-hidden="true" className="size-icon-sm" />
                        <span className="sr-only">{l10n.getString('assistant-checked')}</span>
                      </span>
                    ) : null}
                  </span>
                )}
              </span>
            </button>
          </li>
        );
      })}
    </ul>
  );
}
