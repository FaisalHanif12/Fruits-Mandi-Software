// One known one-time person in the list of a name or phone field (PRD FR-7.29): his name in the
// app language, his phone and the day of his last line, and his name in the other script, so two
// people with the same name are easy to tell apart. Names go through displayName (F38), and the
// phone and the date are isolated numbers (F25).

import type { PukhtaPerson } from '../../../bindings/PukhtaPerson';
import { isoToDisplayDate } from '../../../shared/format';
import { displayName, otherScriptLang, otherScriptName, useLanguage } from '../../../shared/i18n';
import { Code } from '../../../shared/ui';

export function PukhtaPersonRow({ person }: { person: PukhtaPerson }) {
  const language = useLanguage();
  return (
    <span className="flex w-full min-w-0 flex-col">
      <span className="flex items-baseline justify-between gap-3">
        <span className="type-field truncate">{displayName(person, language)}</span>
        {person.phone === '' ? null : <Code>{person.phone}</Code>}
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
