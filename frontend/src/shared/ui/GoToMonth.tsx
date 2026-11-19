// "مہینے پر جائیں" (PRD FR-19.11, FR-17.12, plan 76): a box above a list or statement that shows one
// month at a time. The Owner types a month, with the shortcuts of `monthOfText` ("10", "10-26",
// "10-2026", or a whole date), and presses Enter or leaves the box: the window of that month opens. A
// month that is not real, after this month or before the first month with entries is refused in
// words next to the box and nothing moves. The check is a hint only (Code Standards 4.1, layer 1):
// the backend refuses a month it does not know as well. The words of the box are shared
// (`common-goto-month`); a screen may give its own.

import { Localized, useLocalization } from '@fluent/react';
import { useState, type ReactNode } from 'react';
import { displayMonth, monthOf, monthOfText } from '../format';
import { useFormNavigation } from '../keyboard';
import { Field } from './Field';
import { NumberTextInput } from './NumberTextInput';

interface GoToMonthProps {
  /** The backend's today, `YYYY-MM-DD`, or null until it is known. */
  today: string | null;
  /** The first month with entries, `YYYY-MM`, or null while it is not known. */
  first: string | null;
  /** The month to go to, `YYYY-MM`, never after this month. */
  onGo: (month: string) => void;
  label?: ReactNode;
  info?: ReactNode;
}

type Problem = 'invalid' | 'future' | 'before' | null;

export function GoToMonth({ today, first, onGo, label, info }: GoToMonthProps) {
  const { l10n } = useLocalization();
  const [text, setText] = useState('');
  const [problem, setProblem] = useState<Problem>(null);

  const commit = () => {
    const typed = text.trim();
    if (today === null || typed === '') return;
    const month = monthOfText(typed, today);
    if (month === null) {
      setProblem('invalid');
      return;
    }
    if (month > monthOf(today)) {
      setProblem('future');
      return;
    }
    if (first !== null && month < first) {
      setProblem('before');
      return;
    }
    setProblem(null);
    setText('');
    onGo(month);
  };

  // Enter in the box is Enter on its last field: it goes to the month (a form's Enter, FR-7.22).
  const navigation = useFormNavigation(commit);

  let error: ReactNode;
  if (problem === 'invalid') {
    error = <Localized id="err-month-invalid" />;
  } else if (problem === 'future' && today !== null) {
    error = <Localized id="err-month-in-future" vars={{ month: displayMonth(monthOf(today)) }} />;
  } else if (problem === 'before' && first !== null) {
    error = <Localized id="err-month-before-first" vars={{ month: displayMonth(first) }} />;
  }

  return (
    <div {...navigation}>
      <Field
        label={label ?? <Localized id="common-goto-month" />}
        info={info ?? <Localized id="common-goto-month-hint" />}
        error={error}
      >
        {(control) => (
          <div onBlur={commit} className="w-filter shrink-0">
            {/* Plain text: the + and - keys of a date box mean next and previous day, not here. */}
            <NumberTextInput
              {...control}
              grouping={undefined}
              inputMode="text"
              placeholder={l10n.getString('common-goto-month-placeholder')}
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
