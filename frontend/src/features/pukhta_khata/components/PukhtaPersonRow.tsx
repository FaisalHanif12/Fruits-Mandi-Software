// One known one-time person in the list of a name or phone field (PRD FR-7.29): his name in the
// app language, his phone and the day of his last line, and his name in the other script, so two
// people with the same name are easy to tell apart. Names go through displayName (F38), and the
// phone and the date are isolated numbers (F25). A person in the dead amount carries the tag "ڈوبی"
// (PRD FR-10.10).

import type { PukhtaPerson } from '../../../bindings/PukhtaPerson';
import { isoToDisplayDate } from '../../../shared/format';
import { displayName, otherScriptLang, otherScriptName, useLanguage } from '../../../shared/i18n';
import { Localized } from '@fluent/react';
import { Code } from '../../../shared/ui';

/** A person of the list. `is_dead` is there only in the list of a field that takes dead people. */
type ListedPerson = Omit<PukhtaPerson, 'is_dead'> & { is_dead?: boolean };

export function PukhtaPersonRow({ person }: { person: ListedPerson }) {
  const language = useLanguage();
  return (
    <span className="flex w-full min-w-0 flex-col">
      <span className="flex items-baseline justify-between gap-3">
        <span className="type-field truncate">{displayName(person, language)}</span>
        <span className="flex shrink-0 items-baseline gap-2">
          {person.is_dead === true ? (
            <Localized id="dead-tag">
              <span className="type-chip rounded-1 bg-warn-soft px-2 text-warn" />
            </Localized>
          ) : null}
          {person.phone === '' ? null : <Code>{person.phone}</Code>}
        </span>
      </span>
      <span className="flex items-baseline justify-between gap-3 type-body text-ink-muted">
        <span lang={otherScriptLang(language)} dir="auto" className="truncate">
          {otherScriptName(person, language)}
        </span>
        <Code>{isoToDisplayDate(person.last_date)}</Code>
      </span>
    </span>
  );
}
