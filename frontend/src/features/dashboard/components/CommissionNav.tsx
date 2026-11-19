// The ways to move the period of the commission page (PRD FR-9.3, FR-19.11, plan 77): under the table, the
// buttons and the "go to" box of the period on screen. The month has `MonthNav` and `GoToMonth` (plan 76); this
// file adds the same pair for one date and for a year, and the date box of a range. They only move what is
// asked for (`YYYY-MM-DD`, `YYYY`); the backend reads the days. The checks here are hints only (Code
// Standards 4.1, layer 1): a date that is not real, one after today or a year after this one is refused
// in words next to the box and nothing moves; the backend refuses them as well. There is no money here.

import { Localized, useLocalization } from '@fluent/react';
import { useState, type ReactNode } from 'react';
import { addDays, dayOfText, isoToDisplayDate } from '../../../shared/format';
import { SHORTCUTS, useFormNavigation } from '../../../shared/keyboard';
import {
  Button,
  CodeInput,
  DateInput,
  Field,
  GoToDate,
  RUNNING_NUMBER,
  WindowNav,
} from '../../../shared/ui';
import { addYears, isIsoYear, yearOf } from '../commission-search';

/** A year typed as two digits is of this century ("26" is 2026). */
const CENTURY = '20';

/** The year a typed text means, or null: four digits, or two (this century). */
function yearOfText(text: string): string | null {
  const typed = text.trim();
  if (/^\d{2}$/.test(typed)) return `${CENTURY}${typed}`;
  return isIsoYear(typed) ? typed : null;
}

interface YearNavProps {
  /** The year shown, `YYYY`. */
  year: string;
  /** Today, `YYYY-MM-DD`, as the backend says it: the page never goes past its year. */
  today: string;
  /** The first year with commission, or null while it is not known. */
  first: string | null;
  onMove: (year: string) => void;
}

/** The previous and next year buttons, with the year between them (Page Up and Page Down are the page's). */
export function YearNav({ year, today, first, onMove }: YearNavProps) {
  const { l10n } = useLocalization();
  const latest = yearOf(today);
  const canNearer = year < latest;
  const canBack = first === null || year > first;
  return (
    <WindowNav
      label={l10n.getString('period-year-nav-label')}
      earlier={<Localized id="period-year-earlier" />}
      earlierShortcut={SHORTCUTS.pastEarlier}
      earlierDetail={<span className="type-body">{addYears(year, -1)}</span>}
      onEarlier={() => {
        if (canBack) onMove(addYears(year, -1));
      }}
      {...(canBack ? {} : { earlierOffNote: <Localized id="period-year-first-note" /> })}
      later={<Localized id="period-year-later" />}
      laterShortcut={SHORTCUTS.pastLater}
      laterDetail={<span className="type-body">{addYears(year, 1)}</span>}
      onLater={() => {
        if (canNearer) onMove(addYears(year, 1));
      }}
      {...(canNearer ? {} : { laterOffNote: <Localized id="period-year-latest-note" /> })}
      middle={
        <>
          <Localized id="period-year-title" vars={{ year }} elems={RUNNING_NUMBER}>
            <span className="type-body text-ink" />
          </Localized>
          {canNearer ? (
            <Button
              variant="ghost"
              onClick={() => {
                onMove(latest);
              }}
            >
              <Localized id="period-year-latest-button" />
            </Button>
          ) : null}
        </>
      }
    />
  );
}

interface DayNavProps {
  /** The day shown, `YYYY-MM-DD`. */
  day: string;
  today: string;
  onMove: (day: string) => void;
}

/** The previous and next day buttons, with the day between them. The later button stops at today. */
export function DayNav({ day, today, onMove }: DayNavProps) {
  const { l10n } = useLocalization();
  const canNearer = day < today;
  return (
    <WindowNav
      label={l10n.getString('period-date-nav-label')}
      earlier={<Localized id="period-date-earlier" />}
      earlierShortcut={SHORTCUTS.pastEarlier}
      earlierDetail={<span className="type-body">{isoToDisplayDate(addDays(day, -1))}</span>}
      onEarlier={() => {
        onMove(addDays(day, -1));
      }}
      later={<Localized id="period-date-later" />}
      laterShortcut={SHORTCUTS.pastLater}
      laterDetail={<span className="type-body">{isoToDisplayDate(addDays(day, 1))}</span>}
      onLater={() => {
        if (canNearer) onMove(addDays(day, 1));
      }}
      {...(canNearer ? {} : { laterOffNote: <Localized id="period-date-latest-note" /> })}
      middle={
        <>
          <span className="type-body text-ink">
            <Localized
              id="period-date-title"
              vars={{ date: isoToDisplayDate(day) }}
              elems={RUNNING_NUMBER}
            >
              <span />
            </Localized>
          </span>
          {canNearer ? (
            <Button
              variant="ghost"
              onClick={() => {
                onMove(today);
              }}
            >
              <Localized id="period-date-latest-button" />
            </Button>
          ) : null}
        </>
      }
    />
  );
}

interface GoToYearProps {
  today: string | null;
  first: string | null;
  onGo: (year: string) => void;
}

type YearProblem = 'invalid' | 'future' | 'before' | null;

/** "سال پر جائیں": a box that takes a year, "2026" or "26", on Enter or on leaving it (as `GoToMonth`). */
export function GoToYear({ today, first, onGo }: GoToYearProps) {
  const { l10n } = useLocalization();
  const [text, setText] = useState('');
  const [problem, setProblem] = useState<YearProblem>(null);
  const commit = () => {
    const typed = text.trim();
    if (today === null || typed === '') return;
    const year = yearOfText(typed);
    if (year === null) {
      setProblem('invalid');
      return;
    }
    if (year > yearOf(today)) {
      setProblem('future');
      return;
    }
    if (first !== null && year < first) {
      setProblem('before');
      return;
    }
    setProblem(null);
    setText('');
    onGo(year);
  };
  const navigation = useFormNavigation(commit);
  let error: ReactNode;
  if (problem === 'invalid') {
    error = <Localized id="err-year-invalid" />;
  } else if (problem === 'future' && today !== null) {
    error = <Localized id="err-year-in-future" vars={{ year: yearOf(today) }} />;
  } else if (problem === 'before' && first !== null) {
    error = <Localized id="err-year-before-first" vars={{ year: first }} />;
  }
  return (
    <div {...navigation}>
      <Field
        label={<Localized id="period-goto-year" />}
        info={<Localized id="period-goto-year-hint" />}
        error={error}
      >
        {(control) => (
          <div onBlur={commit} className="w-filter shrink-0">
            <CodeInput
              {...control}
              placeholder={l10n.getString('period-goto-year-placeholder')}
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

/** "تاریخ پر جائیں" for the one date of the page: the shared box with the page's own words. */
export function GoToCommissionDate({
  today,
  onGo,
}: {
  today: string | null;
  onGo: (day: string) => void;
}) {
  const { l10n } = useLocalization();
  return (
    <GoToDate
      today={today}
      onGo={onGo}
      label={<Localized id="period-goto-date" />}
      info={<Localized id="period-goto-date-hint" />}
      placeholder={l10n.getString('common-goto-day-placeholder')}
    />
  );
}

interface DateBoxProps {
  label: ReactNode;
  /** The date now, `YYYY-MM-DD`, or null. The box shows it as DD-MM-YYYY. */
  value: string | null;
  today: string | null;
  onCommit: (day: string) => void;
  /** The backend's refusal of this date, shown under the box. */
  refusal?: ReactNode;
}

/**
 * A date of the range (از or تا): the text is typed, and it is taken on Enter or on leaving the box, with the
 * shortcuts of PRD FR-7.19 ("10", "10-9", + and -). Half a date is never sent. The page gives it a `key` of
 * its value, so a date that changed by itself (a kind change) shows again in the box.
 */
export function CommissionDateBox({ label, value, today, onCommit, refusal }: DateBoxProps) {
  const [text, setText] = useState(value === null ? '' : isoToDisplayDate(value));
  const [problem, setProblem] = useState<'invalid' | 'future' | null>(null);
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
    setText(isoToDisplayDate(day));
    if (day !== value) onCommit(day);
  };
  let error: ReactNode = refusal;
  if (problem === 'invalid') {
    error = <Localized id="err-date-invalid" />;
  } else if (problem === 'future' && today !== null) {
    error = <Localized id="err-date-in-future" vars={{ today: isoToDisplayDate(today) }} />;
  }
  return (
    <Field label={label} error={error}>
      {(control) => (
        <div onBlur={commit}>
          <DateInput
            {...control}
            value={text}
            onChange={(next) => {
              setText(next);
              setProblem(null);
            }}
          />
        </div>
      )}
    </Field>
  );
}
