// The quality box (درجہ, PRD FR-12.42) of the screens that filter sale lines: the sales registers R2, R3
// and R4 and the analytics view A. One box for every screen that has it, so the words are the same: the
// three qualities are the glossary's (`GRADE_WORD_ID`), the same as on the sale line and the prints.
// Choosing "all qualities" sends no filter, so a line with no quality is still counted.

import { Localized, useLocalization } from '@fluent/react';
import type { ReactNode } from 'react';
import { GRADE_CHOICES, GRADE_WORD_ID, type GradeChoice } from '../format';
import { Field } from './Field';
import { Select } from './Select';

interface GradeSelectProps {
  value: GradeChoice;
  onChange: (value: GradeChoice) => void;
  /** The backend's refusal for this box, shown under it. */
  error?: ReactNode;
}

export function GradeSelect({ value, onChange, error }: GradeSelectProps) {
  const { l10n } = useLocalization();
  return (
    <Field label={<Localized id="common-grade" />} error={error}>
      {(control) => (
        <Select<GradeChoice>
          {...control}
          value={value}
          onChange={onChange}
          options={GRADE_CHOICES.map((choice) => ({
            value: choice,
            label: l10n.getString(choice === 'all' ? 'common-grade-all' : GRADE_WORD_ID[choice]),
          }))}
        />
      )}
    </Field>
  );
}
