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
import { useNavigate, useSearch } from '@tanstack/react-router';
import { useCallback, useMemo, useState } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import type { UiError } from '../../../bindings/UiError';
import type { UiErrorItem } from '../../../bindings/UiErrorItem';
import { uiErrorOf } from '../../../shared/api';
import { OpenOnItsScreen, finishFixSession } from '../../../shared/edit';
import { pickedOf } from '../../../shared/accounts';
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
  fieldMark,
  type PickerValue,
} from '../../../shared/ui';
import { useVoucherEditable, vouchersPreview } from '../../vouchers';
import { GeneralLinesSection } from '../components/GeneralLinesSection';
import { GeneralListSection } from '../components/GeneralListSection';
import { GeneralPeople } from '../components/GeneralPeople';
import { GeneralSummary } from '../components/GeneralSummary';
import { EntryFrame } from '../components/EntryFrame';
import { EntryNotes } from '../components/EntryNotes';
import {
  ClosedCashEntry,
  VoucherEditBoxes,
  VoucherEditTop,
  isReadyToEdit,
  useVoucherEdit,
} from '../edit-voucher';
import { generalSeedOf, useEditDraft, type GeneralSeed } from '../edit-seed';
import { previewRefusal } from '../entry-support';
import { journalMarkCells } from '../edit-marks';
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
import { parseGeneralSearch } from '../screen-type';
import type { VoucherEditable, VoucherEntry } from '../types';

/** The address of this screen. */
const GENERAL_PATH = '/rokar/general';
/** Problems that wait until the user tries to save: the summary already says the sides differ. */
const HELD_BACK: ReadonlySet<string> = new Set([
  'ERR_GENERAL_TOO_FEW_LINES',
  'ERR_GENERAL_NOT_BALANCED',
]);

/**
 * The grid's store for a new entry. With a party (plan 49, PRD FR-7.79) his account is chosen on the
 * first line, with no amount on either side; nothing else is filled.
 */
function newStore(party: PickerValue | null) {
  const store = createEntryStore(Array.from({ length: GENERAL_START_LINES }, () => ({})));
  const first = store.getState().rows[0];
  if (party !== null && first !== undefined) {
    store
      .getState()
      .updatePicked(first.rowId, GENERAL_ACCOUNT, pickedOf({ ...party, code: party.code ?? '' }));
  }
  return store;
}

/** The grid's store filled with the lines of a saved entry, each with its account (plan 44). */
function seededStore(seed: GeneralSeed) {
  const store = createEntryStore(seed.lines.map((line) => line.typed));
  const { rows, updatePicked } = store.getState();
  seed.lines.forEach((line, index) => {
    const row = rows[index];
    if (row !== undefined && line.account !== null)
      updatePicked(row.rowId, GENERAL_ACCOUNT, line.account);
  });
  return store;
}

/** A refusal made of the problems at one line's field, for the field under the grid. */
function refusalOf(items: readonly UiErrorItem[]): UiError | null {
  return items.length === 0 ? null : { ok: false, errors: [...items], ref: '' };
}

interface EditorProps {
  initialDate: string;
  /** Today, `YYYY-MM-DD`, for the list of saved entries. */
  today: string;
  /** The entry was saved or abandoned: a new, empty one starts, on the same day. */
  onFinished: (date: string) => void;
  /** The saved voucher open for edit (plan 44); null for a new entry. */
  editable?: VoucherEditable | null;
  /** The lines of the saved entry, to fill the grid. */
  seed?: GeneralSeed | null;
  /** The account a new entry starts with, on its first line (plan 49, PRD FR-7.79). */
  party?: PickerValue | null;
}

function GeneralEditor({
  initialDate,
  today,
  onFinished,
  editable = null,
  seed = null,
  party = null,
}: EditorProps) {
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  const language = useLanguage();
  const errorText = useErrorText();
  const formRef = useFocusFirstField<HTMLFormElement>();
  const [store] = useState(() => (seed === null ? newStore(party) : seededStore(seed)));
  const rows = store((state) => state.rows);
  const focus = store((state) => state.focus);
  const [date, setDate] = useState(initialDate);
  const [narration, setNarration] = useState(seed?.narration ?? '');
  const [people, setPeople] = useState<PeopleMap>(() => {
    // The one-time person of each saved line is chosen again, as when it was typed.
    const chosen: Record<string, PersonChoice> = {};
    seed?.lines.forEach((line, index) => {
      const row = store.getState().rows[index];
      if (row !== undefined && line.person !== null) {
        chosen[row.rowId] = { text: displayName(line.person, language), person: line.person };
      }
    });
    return chosen;
  });
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
  // The entry as the backend's review reads it (plan 44), once there are lines.
  const reviewEntry = useMemo<VoucherEntry | null>(
    () =>
      sent.lines.length === 0
        ? null
        : { type: 'general', ...buildGeneral(date, narration, sent.lines, false) },
    [sent, date, narration],
  );
  const edit = useVoucherEdit(editable, reviewEntry);
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
    save: (input) =>
      edit === null ? saveGeneral.mutateAsync(input) : edit.send({ type: 'general', ...input }),
    // An edit asks the same function as the save, with the entry it replaces taken out first.
    freshCash: (input) =>
      edit === null
        ? generalPreview(buildGeneralPreview(input.date, input.lines)).then((fresh) => fresh.cash)
        : vouchersPreview({ type: 'general', ...input }, edit.editing.row.id).then(
            (fresh) => fresh.cash,
          ),
    cash: preview?.cash,
    current,
    onSaved: (saved) => {
      const number = l10n.getString(`vouchers-number-${GENERAL_KIND}`, {
        number: String(saved.number),
      });
      if (edit !== null) {
        // An edit is done: the days reopened for it close again and a new entry starts (plan 44).
        showToast('success', l10n.getString('cashbook-edited', { number }));
        void finishFixSession(l10n);
        void navigate({ to: GENERAL_PATH });
        return;
      }
      showToast('success', l10n.getString('cashbook-saved', { kind: GENERAL_KIND, number }));
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

  // The cells of the lines that an earlier edit changed (PRD FR-7.71). The lines of the entry are the
  // lines sent, in order: line i of the history is the i-th row that was not blank.
  const cellMarks = journalMarkCells(edit?.marks, sent.rowIds, rows);

  const cardAccount =
    editorCard ??
    (focus === null
      ? null
      : (rows.find((row) => row.rowId === focus.rowId)?.picked?.[GENERAL_ACCOUNT]?.id ?? null));

  const canSave = current && preview?.can_save === true;
  const save = async (): Promise<boolean> => {
    setTried(true);
    if (!canSave) return false;
    // An entry of an earlier day asks why it is changed, once (PRD FR-7.70).
    if (edit?.needsReason() === true) {
      edit.setAskReason(true);
      return false;
    }
    return flow.save();
  };
  const startNew = () => {
    if (edit !== null) {
      void finishFixSession(l10n);
      void navigate({ to: GENERAL_PATH });
      return;
    }
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
      top={edit === null ? undefined : <VoucherEditTop edit={edit} />}
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
      below={edit === null ? <GeneralListSection today={today} /> : null}
      between={
        <GeneralLinesSection
          store={store}
          people={people}
          calc={calc}
          cellErrors={cellErrors}
          cellMarks={cellMarks}
          cardAccount={cardAccount}
          onCardAccount={setEditorCard}
          side={
            <>
              {edit === null ? null : (
                <VoucherEditBoxes
                  edit={edit}
                  busy={flow.busy}
                  onReason={() => {
                    void flow.save();
                  }}
                  onCancelled={() => {
                    void finishFixSession(l10n);
                    void navigate({ to: GENERAL_PATH });
                  }}
                />
              )}
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
          mark={fieldMark(edit?.marks, 'narration')}
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

/** A saved journal entry open for edit (`?id=`): it waits for the entry and its lines, then edits it. */
function GeneralEdit({ id, today }: { id: number; today: string }) {
  const editable = useVoucherEditable(id);
  const isJournal = editable.data === undefined || editable.data.entry.type === 'general';
  const ready = editable.data !== undefined && isReadyToEdit(editable.data.row);
  const seed = useEditDraft(isJournal && ready ? editable.data : undefined, generalSeedOf);
  // An entry of another screen opens on its own screen.
  if (editable.data !== undefined && !isJournal) return <OpenOnItsScreen row={editable.data.row} />;
  if (editable.data !== undefined && !ready) {
    return <ClosedCashEntry titleId="cashbook-general-screen-title" row={editable.data.row} />;
  }
  if (editable.isError || seed.isError) {
    return (
      <ErrorState
        title={<Localized id="cashbook-load-failed" />}
        error={uiErrorOf(editable.isError ? editable.error : seed.error)}
        onRetry={() => {
          void editable.refetch();
        }}
      />
    );
  }
  if (editable.data === undefined || seed.data === undefined) return <LoadingState />;
  return (
    <GeneralEditor
      key={`${String(id)}-${String(editable.data.row.version)}`}
      initialDate={seed.data.date}
      today={today}
      onFinished={() => undefined}
      editable={editable.data}
      seed={seed.data}
    />
  );
}

/** The route's page: it waits for today, then opens an empty entry on that day. */
export function GeneralPage() {
  const day = useDayStatus();
  const rawSearch: Record<string, unknown> = useSearch({ strict: false });
  const search = parseGeneralSearch(rawSearch);
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
  if (search.id !== undefined) return <GeneralEdit id={search.id} today={day.data.today} />;
  return (
    <GeneralEditor
      key={search.party === undefined ? entry.n : `${String(entry.n)}-${String(search.party.id)}`}
      party={search.party ?? null}
      today={day.data.today}
      initialDate={entry.date ?? isoToDisplayDate(day.data.today)}
      onFinished={(date) => {
        setEntry((previous) => ({ n: previous.n + 1, date }));
      }}
    />
  );
}
