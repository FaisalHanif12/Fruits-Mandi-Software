// The words of an answer, piece by piece (PRD FR-7.74, D6 Section 2). A `text` piece is the model's
// words, written as they are and never drawn as a number. Every other piece is something the app put
// in from the books: a `figure` (large, through `<Num>`, with ✓ and ⓘ), a `name` (through
// `displayName`, with its tag and its code), a `date`, a `month` and a `doc` (a document with its
// number). The panel writes them; it works nothing out (CLAUDE.md rule 1).

import { useLocalization } from '@fluent/react';
import type { AnswerPart } from '../../../bindings/AnswerPart';
import type { ReportCell } from '../../../bindings/ReportCell';
import { isoToDisplayDate } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { Code, Num, ScriptText } from '../../../shared/ui';
import { useListContext } from '../../analytics';
import { cellText, monthText } from '../../reports';
import { Figure } from './Figure';

/** A name with its tag in the app language, and its code after it in brackets (PRD FR-7.74). */
export function NameWithCode({
  name,
}: {
  name: { name_ur: string; name_en: string; tag: string | null; code: string | null };
}) {
  const language = useLanguage();
  return (
    <>
      <ScriptText>{displayName(name, language)}</ScriptText>
      {name.code === null ? null : (
        <>
          {' ('}
          <Code>{name.code}</Code>
          {')'}
        </>
      )}
    </>
  );
}

function PartView({ part }: { part: AnswerPart }) {
  const context = useListContext();
  switch (part.type) {
    case 'text':
      // The model's own words: text as written, never a number cell.
      return <span>{part.text}</span>;
    case 'figure':
      return (
        <Figure cell={part.cell} dir={part.dir} checked={part.checked} working={part.working} />
      );
    case 'name':
      return <NameWithCode name={part.name} />;
    case 'date':
      return <Num inherit>{isoToDisplayDate(part.iso)}</Num>;
    case 'month':
      return <bdi>{monthText(context.text, part.year, part.month)}</bdi>;
    case 'doc': {
      const cell: ReportCell = { type: 'doc', type_id: part.type_id, number: part.number };
      return <bdi>{cellText(cell, context)}</bdi>;
    }
  }
}

export function AnswerText({ parts }: { parts: readonly AnswerPart[] }) {
  const { l10n } = useLocalization();
  if (parts.length === 0) return null;
  return (
    <p aria-label={l10n.getString('assistant-answer-label')} className="type-lead text-ink">
      {parts.map((part, index) => (
        // The pieces of an answer have no id; their order is the answer.
        <PartView key={index} part={part} />
      ))}
    </p>
  );
}
