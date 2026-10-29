// The جنرل screen (route /rokar/general; PRD 16.4, FR-6.5, FR-13.5, FR-6.7, FR-7.67): a journal entry
// of lines with the account, بنام and جمع, for corrections, transfers and goods returns. It saves
// only when the two sides are equal; the backend says so (`general_preview`, the same function as
// the save, Code Standards B5) and the Save button stays off until it does. Lines on income or shop
// expense accounts are split between the partners by the backend; a line on a پختہ کھاتہ account
// names the one-time person; a line that credits روکڑ asks the cash question.
//
// - What is typed lives in the grid's store and in this screen's state, never only in the grid, so
//   a language switch loses nothing (F39).
// - The frontend adds nothing up (CLAUDE.md rule 1): the totals, the difference, the split and the
//   cash are the backend's.
// - Keys, the leave guard and the cash question come from EntryFrame.
// - Four states: the day (loading, error with a retry), the lines (blank rows are the empty state),
//   the totals (waiting, failed with a retry, the answer) and the screen itself.

import { Localized, useLocalization } from '@fluent/react';
import { useCallback, useMemo, useState } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import type { UiError } from '../../../bindings/UiError';
import type { UiErrorItem } from '../../../bindings/UiErrorItem';
import { uiErrorOf } from '../../../shared/api';
import { createEntryStore } from '../../../shared/grid';
import { useEntrySaveFlow, useFocusFirstField } from '../../../shared/hooks';
import { isoToDisplayDate } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import {
  DateInput,
  ErrorState,
  Field,
  FreeTextInput,
  LoadingState,
  fieldError,
  showToast,
  useErrorText,
  FormCard,
} from '../../../shared/ui';
import { GeneralLinesSection } from '../components/GeneralLinesSection';
import { GeneralPeople } from '../components/GeneralPeople';
import { GeneralSummary } from '../components/GeneralSummary';
import { EntryFrame } from '../components/EntryFrame';
import { EntryNotes } from '../components/EntryNotes';
import { previewRefusal } from '../entry-support';
import {
  GENERAL_ACCOUNT,
  GENERAL_FIELD_PATHS,
  GENERAL_KIND,
  GENERAL_PERSON,
  GENERAL_START_LINES,
  buildGeneral,
  buildGeneralPreview,
  generalIsFilled,
  lineProblems,
  sentLines,
  type PeopleMap,
  type PersonChoice,
} from '../general-draft';
import { generalPreview } from '../api';
import { useGeneralPreview, useSaveGeneral } from '../hooks';

/** The address of this screen. */
const GENERAL_PATH = '/rokar/general';
/** Problems that wait until the user tries to save: the summary already says the sides differ. */
const HELD_BACK: ReadonlySet<string> = new Set([
  'ERR_GENERAL_TOO_FEW_LINES',
  'ERR_GENERAL_NOT_BALANCED',
]);

function newStore() {
  return createEntryStore(Array.from({ length: GENERAL_START_LINES }, () => ({})));
}

/** A refusal made of the problems at one line's field, for the field under the grid. */
function refusalOf(items: readonly UiErrorItem[]): UiError | null {
  return items.length === 0 ? null : { ok: false, errors: [...items], ref: '' };
}

interface EditorProps {
  initialDate: string;
  /** The entry was saved or abandoned: a new, empty one starts, on the same day. */
  onFinished: (date: string) => void;
}

function GeneralEditor({ initialDate, onFinished }: EditorProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const errorText = useErrorText();
  const formRef = useFocusFirstField<HTMLFormElement>();
  const [store] = useState(newStore);
  const rows = store((state) => state.rows);
  const focus = store((state) => state.focus);
  const [date, setDate] = useState(initialDate);
  const [narration, setNarration] = useState('');
  const [people, setPeople] = useState<PeopleMap>({});
  const [tried, setTried] = useState(false);
  const [editorCard, setEditorCard] = useState<number | null>(null);

  const sent = useMemo(() => sentLines(rows, people), [rows, people]);
  const filled = generalIsFilled(narration, rows);
  const ask = useMemo(
    () =>
      sent.lines.length === 0
        ? null
        : { input: buildGeneralPreview(date, sent.lines), rowIds: sent.rowIds },
    [sent, date],
  );
  const { query, current } = useGeneralPreview(ask);
  const answer = ask === null ? undefined : query.data;
  const preview = answer?.preview;

  const entryErrors = useMemo(
    () => (preview?.errors ?? []).filter((item) => tried || !HELD_BACK.has(item.code)),
    [preview, tried],
  );
  const previewError = useMemo(() => previewRefusal(entryErrors, tried), [entryErrors, tried]);

  const draftText = useMemo(
    () => JSON.stringify({ date, narration, lines: sent.lines }),
    [date, narration, sent],
  );
  const saveGeneral = useSaveGeneral();
  const flow = useEntrySaveFlow({
    draftText,
    build: (confirm) => buildGeneral(date, narration, sent.lines, confirm),
    save: (input) => saveGeneral.mutateAsync(input),
    freshCash: (input) =>
      generalPreview(buildGeneralPreview(input.date, input.lines)).then((fresh) => fresh.cash),
    cash: preview?.cash,
    current,
    onSaved: (saved) => {
      showToast(
        'success',
        l10n.getString('cashbook-saved', {
          kind: GENERAL_KIND,
          number: l10n.getString(`vouchers-number-${GENERAL_KIND}`, {
            number: String(saved.number),
          }),
        }),
      );
      onFinished(date);
    },
  });
  const error = flow.refusal ?? previewError;

  // The problems of the lines go to their cells: from the save's refusal, else from the preview.
  const lineErrors = useMemo(() => {
    if (flow.refusal !== null) return flow.refusal.errors;
    return (preview?.lines ?? []).flatMap((line) => line.errors);
  }, [flow.refusal, preview]);
  const problems = useMemo(
    () =>
      lineProblems(lineErrors, flow.refusal !== null ? sent.rowIds : (answer?.rowIds ?? []), tried),
    [lineErrors, flow.refusal, sent.rowIds, answer, tried],
  );
  const cellErrors = useMemo(() => {
    const byRow: Record<string, Record<string, string>> = {};
    for (const problem of problems.onCells) {
      const row = (byRow[problem.rowId] ??= {});
      row[problem.cell] ??= errorText(problem.item);
    }
    return byRow;
  }, [problems, errorText]);
  const personErrorOf = useCallback(
    (rowId: string): UiError | null =>
      refusalOf(
        problems.onCells
          .filter((problem) => problem.rowId === rowId && problem.cell === GENERAL_PERSON)
          .map((problem) => ({ ...problem.item, field: 'person_key' })),
      ),
    [problems],
  );

  const calc = useMemo(() => {
    const byRow: Record<string, Record<string, string>> = {};
    for (const row of rows) {
      const person = people[row.rowId]?.person;
      if (person !== null && person !== undefined) {
        byRow[row.rowId] = {
          [GENERAL_PERSON]: [displayName(person, language), person.phone].join(' ').trim(),
        };
      }
    }
    return byRow;
  }, [rows, people, language]);

  const cardAccount =
    editorCard ??
    (focus === null
      ? null
      : (rows.find((row) => row.rowId === focus.rowId)?.picked?.[GENERAL_ACCOUNT]?.id ?? null));

  const canSave = current && preview?.can_save === true;
  const save = async (): Promise<boolean> => {
    setTried(true);
    return canSave ? flow.save() : false;
  };
  const startNew = () => {
    onFinished(date);
  };
  const setPerson = (rowId: string, choice: PersonChoice) => {
    setPeople((previous) => ({ ...previous, [rowId]: choice }));
    setTried(false);
  };

  const summary =
    ask !== null && query.isError ? (
      <ErrorState
        title={<Localized id="cashbook-preview-failed" />}
        error={uiErrorOf(query.error)}
        onRetry={() => {
          void query.refetch();
        }}
      />
    ) : ask !== null && preview === undefined ? (
      <LoadingState rows={3} />
    ) : (
      <GeneralSummary preview={preview} />
    );

  const shownPaths = [
    ...GENERAL_FIELD_PATHS,
    ...problems.onCells.map((problem) => problem.item.field ?? ''),
  ];

  return (
    <EntryFrame<string>
      titleId="cashbook-general-screen-title"
      helpId="cashbook-general-screen-help"
      path={GENERAL_PATH}
      filled={filled}
      saveBlocked={!canSave}
      busy={flow.busy}
      onSave={save}
      onNew={startNew}
      flow={flow}
      error={error}
      fieldsShown={shownPaths}
      formRef={formRef}
      below={null}
      between={
        <GeneralLinesSection
          store={store}
          people={people}
          calc={calc}
          cellErrors={cellErrors}
          cardAccount={cardAccount}
          onCardAccount={setEditorCard}
          side={
            <>
              {summary}
              <EntryNotes warnings={preview?.warnings ?? []} cash={undefined} />
              <GeneralPeople
                rows={rows}
                people={people}
                onChange={setPerson}
                onPicked={(rowId) => {
                  const index = rows.findIndex((row) => row.rowId === rowId);
                  store.getState().setPendingFocus({ rowIndex: Math.max(index, 0), field: 'dr' });
                }}
                errorOf={personErrorOf}
              />
            </>
          }
        />
      }
    >
      <FormCard>
        <Field label={<Localized id="cashbook-field-date" />} error={fieldError(error, 'date')}>
          {(props) => (
            <DateInput
              {...props}
              value={date}
              onChange={(next) => {
                setDate(next);
                setTried(false);
              }}
            />
          )}
        </Field>
        <Field
          label={<Localized id="cashbook-field-narration" />}
          error={fieldError(error, 'narration')}
        >
          {(props) => (
            <FreeTextInput
              {...props}
              value={narration}
              onChange={(next) => {
                setNarration(next);
                setTried(false);
              }}
            />
          )}
        </Field>
      </FormCard>
    </EntryFrame>
  );
}

/** The route's page: it waits for today, then opens an empty entry on that day. */
export function GeneralPage() {
  const day = useDayStatus();
  const [entry, setEntry] = useState<{ n: number; date: string | null }>({ n: 0, date: null });
  if (day.isPending) return <LoadingState />;
  if (day.isError) {
    return (
      <ErrorState
        title={<Localized id="cashbook-load-failed" />}
        error={uiErrorOf(day.error)}
        onRetry={() => {
          void day.refetch();
        }}
      />
    );
  }
  return (
    <GeneralEditor
      key={entry.n}
      initialDate={entry.date ?? isoToDisplayDate(day.data.today)}
      onFinished={(date) => {
        setEntry((previous) => ({ n: previous.n + 1, date }));
      }}
    />
  );
}
