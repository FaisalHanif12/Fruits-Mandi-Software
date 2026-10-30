// The form of ONE lot's sale lines (plan 16): a later sale of the goods left in a lot (PRD
// FR-12.17) and the edit of a later posting (FR-12.18). The lot is fixed and shown read-only; only
// the sale lines are typed. It is the sale book without the header and the lots: the same grid of
// lines (buyer, qty, rate, the amounts the backend works out, each with its ⓘ), the details of the
// line that has focus, the party card, the live summary and partner split from the backend's
// preview, the cash note, the problems in words, and F10 to save.
//
// - The preview and the save are the backend's one function (B5): this form only chooses which
//   command (`source`, `mutation`). It works out no money and no quantity (CLAUDE.md rule 1).
// - Everything typed is in the stores above the grid, so a language switch loses nothing (F39).
// - A later sale does not charge the lot's freight, labour and the rest again; the form says so in
//   one line (PRD FR-12.17) and the summary shows only the new lines.
// - The cash box (PRD FR-6.7) opens before a save that takes cash below zero.

import { Localized, useLocalization } from '@fluent/react';
import { FileText } from 'lucide-react';
import { useMemo, useRef, useState, type ReactNode } from 'react';
import { LeaveGuard } from '../../../app/LeaveGuard';
import type { FieldError } from '../../../bindings/FieldError';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate, useGrouping } from '../../../shared/format';
import { useRefusalIntoView } from '../../../shared/hooks';
import { SHORTCUTS, useEscape, useShortcut } from '../../../shared/keyboard';
import type { EntryRow } from '../../../shared/grid';
import {
  Button,
  CashConfirmDialog,
  CashNote,
  EmptyState,
  ErrorNotice,
  ErrorState,
  LoadingState,
  PartyCard,
  ReasonDialog,
  useErrorText,
  InfoHint,
} from '../../../shared/ui';
import { useBikriDraft } from '../draft-store';
import { buildGridViews } from '../grid-views';
import { useBikriPreview, type PreviewSource } from '../hooks/useBikriPreview';
import { useDraftFills } from '../hooks/useDraftFills';
import type { EditReason } from '../hooks/useEditReason';
import { useSaveFlow, type SaveMutation } from '../hooks/useSaveFlow';
import { problemsOf, type Problem } from '../problems';
import { SALE, type BikriDraft, type SummaryTotals } from '../types';
import { ActiveLine, SalesSection } from './SalesSection';
import { ProblemList } from './ProblemList';
import { SplitPanel } from './SplitPanel';
import { SummaryPanel } from './SummaryPanel';

/** The field path of the day of a later sale in the backend's errors. */
const DAY_FIELD = 'date';

/** The problems that stop a save, as one refusal, so each field can show the words of its own. */
function errorOf(problems: readonly Problem[]): UiError | null {
  const errors = problems.filter((problem) => problem.severity === 'error');
  return errors.length === 0
    ? null
    : { ok: false, errors: errors.map((problem) => problem.item), ref: '' };
}

/** The problems of a refusal that point at a field, in the shape of the preview's problems. */
function refusalErrors(error: UiError | null): FieldError[] {
  if (error === null) return [];
  return error.errors
    .filter((item) => item.field !== null && item.field !== '')
    .map((item) => ({ code: item.code, field: item.field ?? '', values: item.values }));
}

function withoutRepeats(items: readonly FieldError[]): FieldError[] {
  const seen = new Set<string>();
  return items.filter((item) => {
    const key = `${item.code}|${item.field}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

/** The lines of the draft, to tell whether anything was typed. */
function hasLines(draft: BikriDraft): boolean {
  return (
    draft.lots[0]?.sales.some(
      (sale) => sale.buyer_account_id !== null || sale.seller_rate !== '',
    ) ?? false
  );
}

interface LinesFormProps<R extends { number: number }> {
  /** The row id of the one lot row, as the preview echoes it. */
  lotKey: string;
  source: PreviewSource;
  mutation: SaveMutation<R>;
  /** The message after a save. It gets `$number`. */
  toastId: string;
  onSaved: (saved: R) => void;
  /** The title, the help line and the card of the lot above the lines. */
  title: ReactNode;
  help: ReactNode;
  children?: ReactNode;
  /**
   * The field or fact of the posting day. A function gets the problems of the screen, so the day
   * field shows its own (a date error is on the field `date`, plan 16 contract).
   */
  dayField: ReactNode | ((error: UiError | null) => ReactNode);
  /** Words under the lines grid (what changed, in an edit). Gets the preview's numbers of the lot. */
  below?: ((after: SummaryTotals | null) => ReactNode) | undefined;
  /** Cells the user changed, marked in the grid (an edit). */
  cellChanged?: Record<string, Record<string, string>> | undefined;
  /** Is something typed or changed that is not saved? Default: some line has a buyer or a rate. */
  dirty?: boolean | undefined;
  /**
   * An edit of an earlier day asks for a reason before it saves (plan 16, open questions). The
   * reason is kept in `store`, which the save reads. `day` is the day of the posting.
   */
  reason?: { store: EditReason; needed: boolean; day: string } | undefined;
  /** The label of the save button (Fluent ID). */
  saveId: string;
  /** Leave the screen (Esc, and the Back button). */
  onBack: () => void;
  /** Can the screen be saved at all? A closed day cannot (an edit shows why). */
  canSave: boolean;
  /** A notice above the lines (a closed day). */
  notice?: ReactNode;
  /** Can Ctrl+Delete remove this line? */
  canRemoveRow?: ((row: EntryRow) => boolean) | undefined;
}

export function LinesForm<R extends { number: number }>({
  lotKey,
  source,
  mutation,
  toastId,
  onSaved,
  title,
  help,
  children,
  dayField,
  below,
  cellChanged,
  dirty,
  reason,
  saveId,
  onBack,
  canSave,
  notice,
  canRemoveRow,
}: LinesFormProps<R>) {
  const { l10n } = useLocalization();
  const grouping = useGrouping();
  const stores = useBikriDraft();
  const errorText = useErrorText();
  const { query, stale, draft } = useBikriPreview(stores, source);
  const preview = query.data;
  useDraftFills(stores, preview, stale);

  const flow = useSaveFlow({
    stores,
    draft,
    preview,
    current: preview !== undefined && !stale,
    mutation,
    askPreview: source.run,
    toastId,
    onSaved: (saved) => {
      onSaved(saved);
    },
    // A "no" at the cash box puts the cursor back on the lines.
    onCashDeclined: () => {
      stores.focusSaleLine(lotKey, 0, SALE.qty);
    },
  });
  const [askReason, setAskReason] = useState(false);
  const dialogOpen = flow.ask !== null || askReason;
  const unsaved = dirty ?? hasLines(draft);
  const trySave = () => {
    if (!canSave) return;
    if (reason?.needed === true && reason.store.get().trim() === '') {
      setAskReason(true);
      return;
    }
    void flow.save(false);
  };
  useShortcut(SHORTCUTS.save, trySave, { enabled: !dialogOpen });
  useEscape('leave', onBack);
  // The backend asks for a reason although the screen did not: ask now.
  const refusedForReason =
    flow.refusal?.errors.some((item) => item.code === 'ERR_REASON_REQUIRED') ?? false;
  const [askedFor, setAskedFor] = useState<UiError | null>(null);
  if (refusedForReason && reason !== undefined && flow.refusal !== askedFor) {
    setAskedFor(flow.refusal);
    setAskReason(true);
  }

  const salesStore = stores.salesFor(lotKey);
  const salesRows = salesStore((state) => state.rows);
  const salesFocus = salesStore((state) => state.focus);
  const activeLine = Math.max(
    salesRows.findIndex((row) => row.rowId === salesFocus?.rowId),
    0,
  );
  const refused = flow.refusal;
  const rootRef = useRef<HTMLDivElement>(null);
  // A refusal above the grid he is typing in is brought into view (never left unseen).
  useRefusalIntoView(rootRef, refused);
  const problems = useMemo(
    () =>
      problemsOf(
        withoutRepeats([...(preview?.errors ?? []), ...refusalErrors(refused)]),
        preview?.warnings ?? [],
        flow.tried,
      ),
    [preview, refused, flow.tried],
  );
  const formError = useMemo(() => errorOf(problems), [problems]);
  const say = useMemo(
    () => (id: string, vars?: Record<string, string>) => l10n.getString(id, vars),
    [l10n],
  );
  const views = buildGridViews({
    preview,
    problems,
    lotKeys: [lotKey],
    selectedLot: lotKey,
    lineKeysByLot: [salesRows.map((row) => row.rowId)],
    showAll: false,
    lotLabel: () => '',
    grouping,
    say,
    words: (problem) => errorText(problem.item),
  });

  // The party card follows the line the cursor is on.
  const [editorCard, setEditorCard] = useState<number | null>(null);
  const focusedLine = salesRows.find((row) => row.rowId === salesFocus?.rowId);
  const cardAccount = editorCard ?? focusedLine?.picked?.[SALE.buyer]?.id ?? null;

  const lotPreview = preview?.lots.find((lot) => lot.key === lotKey) ?? preview?.lots[0];
  const usesWeight =
    lotPreview?.sold_by_weight === true ||
    salesRows.some((row) => (row.typed[SALE.weight] ?? '') !== '');

  const numbers =
    preview === undefined ? null : (
      <>
        <CashNote cash={preview.cash} />
        <SummaryPanel
          totals={lotPreview?.totals ?? null}
          whole={false}
          lotNumber={1}
          netIsNegative={lotPreview?.net_is_negative ?? false}
        />
        <SplitPanel preview={preview} />
      </>
    );

  let numbersBody: ReactNode = numbers;
  if (query.isError) {
    numbersBody = (
      <ErrorState
        title={<Localized id="bikri-preview-failed" />}
        error={uiErrorOf(query.error)}
        onRetry={() => {
          void query.refetch();
        }}
      />
    );
  } else if (preview === undefined) {
    numbersBody = query.isFetching ? (
      <LoadingState rows={6} />
    ) : (
      <EmptyState
        icon={FileText}
        title={<Localized id="bikri-empty-title" />}
        help={<Localized id="bikri-empty-help" />}
      />
    );
  }

  return (
    <div ref={rootRef} className="flex flex-col gap-3 short:gap-2">
      {/* The name with its help behind an ⓘ, and Back and Save on the same row, so the buttons are in
          view whatever the length of the lines below (plan 38, later sale). */}
      <div className="flex flex-wrap items-center justify-between gap-x-4 gap-y-2">
        <div className="flex items-center gap-1">
          <h1 className="type-title">{title}</h1>
          <InfoHint>
            <p>{help}</p>
          </InfoHint>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <Button shortcut={SHORTCUTS.back} onClick={onBack}>
            <Localized id="common-back" />
          </Button>
          <Button
            variant="primary"
            shortcut={SHORTCUTS.save}
            busy={flow.busy}
            disabled={!canSave}
            onClick={trySave}
          >
            <Localized id={saveId} />
          </Button>
        </div>
      </div>
      {notice}
      {children}
      <div className="flex flex-wrap items-end gap-4">
        {typeof dayField === 'function' ? dayField(formError) : dayField}
      </div>
      {refused === null ? null : (
        <ErrorNotice
          error={refused}
          fieldsShown={[
            ...refusalErrors(refused).map((item) => item.field),
            // A date error is shown under the day field.
            ...(typeof dayField === 'function' ? [DAY_FIELD] : []),
          ]}
        />
      )}
      <ProblemList
        problems={problems}
        selectedLot={0}
        activeLine={activeLine}
        headerShown={false}
      />
      <div className="grid grid-cols-1 gap-3 wide:grid-cols-3">
        <div className="flex min-w-0 flex-col gap-3 wide:col-span-2">
          <SalesSection
            lotKey={lotKey}
            lotIndex={0}
            whole={false}
            view={views.sales}
            onCardAccount={setEditorCard}
            single={{ title: <Localized id="bikri-lines-title" /> }}
            cellChanged={cellChanged}
            canRemoveRow={canRemoveRow}
          />
          <ActiveLine
            lotKey={lotKey}
            lotIndex={0}
            preview={preview}
            error={formError}
            usesWeight={usesWeight}
          />
          {below?.(lotPreview?.totals ?? null)}
        </div>
        <div className="flex min-w-0 flex-col gap-3">
          <PartyCard accountId={cardAccount} />
          {numbersBody}
        </div>
      </div>
      {flow.cashAfter === null ? null : (
        <CashConfirmDialog
          open={flow.ask !== null}
          onOpenChange={flow.onCashOpenChange}
          cashAfter={flow.cashAfter}
          onConfirm={flow.confirmCash}
          onCloseFocus={flow.onCashCloseFocus}
        />
      )}
      {reason === undefined ? null : (
        <ReasonDialog
          open={askReason}
          onOpenChange={setAskReason}
          title={<Localized id="bikri-edit-reason-title" />}
          description={
            <Localized id="bikri-edit-reason-text" vars={{ date: isoToDisplayDate(reason.day) }}>
              <span />
            </Localized>
          }
          reasonLabel={<Localized id="bikri-edit-reason-label" />}
          confirmLabel={<Localized id="bikri-edit-reason-confirm" />}
          confirmVariant="primary"
          error={null}
          busy={flow.busy}
          onConfirm={(text) => {
            reason.store.set(text);
            setAskReason(false);
            void flow.save(false);
          }}
        />
      )}
      <LeaveGuard dirty={unsaved} onSave={() => flow.save(false)} />
    </div>
  );
}
