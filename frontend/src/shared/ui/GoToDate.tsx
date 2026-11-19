// "تاریخ پر جائیں" (PRD 12.7, FR-12.47): a date box with its label and a placeholder, above a list that
// moves by windows of days. The Owner types a day, with the shortcuts of FR-7.19 ("10", "10-9", the +
// and - keys), and presses Enter or leaves the box: the screen goes to that day. A day that is not
// real, or after today, is refused in words next to the box and nothing moves. The words of the
// refusal are the backend's (`err-date-invalid`, `err-date-in-future`), so the screen says the same as
// a saved entry would. The check is a hint only (Code Standards 4.1, layer 1): the backend refuses a
// date after today as well. The screen gives the words of the box (its label, hint and placeholder),
// so each module keeps its own words (F31).

import { Localized } from '@fluent/react';
import { useState, type ReactNode } from 'react';
import { dayOfText, isoToDisplayDate } from '../format';
import { useFormNavigation } from '../keyboard';
import { DateInput } from './DateInput';
import { Field } from './Field';

interface GoToDateProps {
  /** The backend's today, `YYYY-MM-DD`, or null until it is known. */
  today: string | null;
  /** The day to go to, `YYYY-MM-DD`, never after today. */
  onGo: (day: string) => void;
  label: ReactNode;
  /** What the box takes, behind an ⓘ next to its label. */
  info?: ReactNode;
  /** The grey words inside the empty box, already in the app language. */
  placeholder: string;
}

type Problem = 'invalid' | 'future' | null;

export function GoToDate({ today, onGo, label, info, placeholder }: GoToDateProps) {
  const [text, setText] = useState('');
  const [problem, setProblem] = useState<Problem>(null);

  const commit = () => {
    const typed = text.trim();
    if (today === null || typed === '') return;
    const day = dayOfText(typed, today);
    if (day === null) {
      setProblem('invalid');
      return;
    }
    if (day > today) {
      setProblem('future');
      return;
    }
    setProblem(null);
    setText('');
    onGo(day);
  };

  // Enter in the box is Enter on its last field: it goes to the day (a form's Enter, FR-7.22).
  const navigation = useFormNavigation(commit);

  let error: ReactNode;
  if (problem === 'invalid') {
    error = <Localized id="err-date-invalid" />;
  } else if (problem === 'future' && today !== null) {
    error = <Localized id="err-date-in-future" vars={{ today: isoToDisplayDate(today) }} />;
  }

  return (
    <div {...navigation}>
      <Field label={label} info={info} error={error}>
        {(control) => (
          <div onBlur={commit} className="w-filter shrink-0">
            <DateInput
              {...control}
              placeholder={placeholder}
              value={text}
              onChange={(next) => {
                setText(next);
                setProblem(null);
              }}
            />
          </div>
        )}
      </Field>
    </div>
  );
}
