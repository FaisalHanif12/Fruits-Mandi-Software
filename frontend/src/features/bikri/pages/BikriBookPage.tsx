// بکری بک (route /bikri/book; PRD 12.1 to 12.5, plan 12): the munshi types the whole بکری and sees
// every number live. One screen: the header, the lots, the sale lines of the selected lot, the
// summary (per lot and for the whole بکری), the live partner split, the cash note, and the party
// card of the party he is on. Nothing is saved here yet (plan 13).
//
// - The frontend works out no money. It sends the draft as typed to `bikri_preview` 50 ms after
//   the last key and shows what comes back (Code Standards F35, CLAUDE.md rule 1). The preview and
//   the save are the same function in the backend, so what is on screen is what will be saved.
// - Everything typed is in the draft stores above the grids, so a language switch re-creates the
//   grids and loses nothing (F39).
// - Keys: the grids and forms get theirs from shared/grid and shared/keyboard (F27). Enter on the
//   last header field goes to the lots; Enter on the last cell of a lot goes to its sale lines.
// - Four states: loading while today arrives, an error with a retry, the empty summary with its
//   button before anything is typed, and the data.

import { Localized, useLocalization } from '@fluent/react';
import { FileText } from 'lucide-react';
import { useEffect, useMemo, useRef, useState } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate, useGrouping } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { isFromStep, useLayoutStep } from '../../../shared/hooks';
import {
  Button,
  EmptyState,
  ErrorState,
  LoadingState,
  PartyCard,
  useErrorText,
} from '../../../shared/ui';
import { CashNote } from '../components/CashNote';
import { HeaderCard } from '../components/HeaderCard';
import { LotMore } from '../components/LotMore';
import { LotsSection } from '../components/LotsSection';
import { ProblemList } from '../components/ProblemList';
import { ActiveLine, SalesSection } from '../components/SalesSection';
import { NumbersStrip } from '../components/NumbersStrip';
import { SplitPanel } from '../components/SplitPanel';
import { SummaryPanel } from '../components/SummaryPanel';
import {
  BikriDraftProvider,
  createBikriDraft,
  useBikriDraft,
  type BikriDraftStores,
} from '../draft-store';
import { isPukhtaSeller } from '../draft';
import { buildGridViews } from '../grid-views';
import { useBikriPreview } from '../hooks/useBikriPreview';
import { useDraftFills } from '../hooks/useDraftFills';
import { LOT_GRID_FIELD, problemsOf, type Problem } from '../problems';
import { LOT, SALE } from '../types';

/** The parts of the screen the keyboard or the mouse can be in, for the party card. */
type Region = 'header' | 'lots' | 'sales';

/** The keys of the lines of one lot, in order. */
function linesOf(stores: BikriDraftStores, lotKey: string): string[] {
  return stores
    .salesFor(lotKey)
    .getState()
    .rows.map((row) => row.rowId);
}

/** A refusal made of the problems that are errors, so each field can show the words of its own. */
function errorOf(problems: readonly Problem[]): UiError | null {
  const errors = problems.filter((problem) => problem.severity === 'error');
  return errors.length === 0
    ? null
    : { ok: false, errors: errors.map((problem) => problem.item), ref: '' };
}

interface EditorProps {
  /** Opens the saved بکری of the same truck. Plan 13 gives it; until then the button is dimmed. */
  onOpenBikri?: ((number: number) => void) | undefined;
}

function Editor({ onOpenBikri }: EditorProps) {
  const { l10n } = useLocalization();
  const grouping = useGrouping();
  const stores = useBikriDraft();
  const errorText = useErrorText();
  const language = useLanguage();
  const { query, stale } = useBikriPreview(stores);
  const preview = query.data;
  useDraftFills(stores, preview, stale);

  const lotRows = stores.lots((state) => state.rows);
  const selected = stores.view((state) => state.selectedLot);
  const whole = stores.view((state) => state.wholeView);
  const zamindar = stores.header((state) => state.zamindar);
  const lotKey = selected ?? lotRows[0]?.rowId ?? '';
  const lotIndex = Math.max(
    lotRows.findIndex((row) => row.rowId === lotKey),
    0,
  );
  const lotRow = lotRows[lotIndex];
  const salesStore = stores.salesFor(lotKey);
  const salesRows = salesStore((state) => state.rows);
  // The whole-sale view draws the lines of every lot, so the page follows the rows of all of them.
  const allRows = stores.all((state) => state.rows);
  const salesFocus = salesStore((state) => state.focus);
  const lotsFocus = stores.lots((state) => state.focus);
  const activeLine = Math.max(
    salesRows.findIndex((row) => row.rowId === salesFocus?.rowId),
    0,
  );

  const problems = useMemo(
    () => (preview === undefined ? [] : problemsOf(preview.errors, preview.warnings, false)),
    [preview],
  );
  const formError = useMemo(() => errorOf(problems), [problems]);
  const say = useMemo(
    () => (id: string, vars?: Record<string, string>) => l10n.getString(id, vars),
    [l10n],
  );
  // The lot of a line in the whole-sale view: its number and its seller (PRD FR-12.1).
  const lotLabel = (index: number) => {
    const seller = lotRows[index]?.picked?.[LOT.seller];
    return seller === undefined
      ? say('bikri-all-lot-no-seller', { lot: String(index + 1) })
      : say('bikri-all-lot', { lot: String(index + 1), seller: displayName(seller, language) });
  };
  const views = buildGridViews({
    preview,
    problems,
    lotKeys: lotRows.map((row) => row.rowId),
    selectedLot: lotKey,
    // Read again whenever any line of any lot changes (`allRows` changed).
    lineKeysByLot: allRows.length === 0 ? [] : lotRows.map((row) => linesOf(stores, row.rowId)),
    showAll: whole,
    lotLabel,
    grouping,
    say,
    words: (problem) => errorText(problem.item),
  });

  // A problem on a field of the "more" panel keeps the panel open; the weight field shows for a
  // lot or line that asks for weight (PRD E23).
  const lotProblems = problems.filter(
    (problem) =>
      problem.where.kind === 'lot' &&
      problem.where.lot === lotIndex &&
      LOT_GRID_FIELD[problem.where.sub] === undefined,
  );
  const usesWeight =
    preview?.lots.find((lot) => lot.key === lotKey)?.sold_by_weight === true ||
    (lotRow?.typed[LOT.weight] ?? '') !== '' ||
    salesRows.some((row) => (row.typed[SALE.weight] ?? '') !== '') ||
    problems.some(
      (problem) =>
        problem.item.code === 'ERR_WEIGHT_REQUIRED' &&
        (problem.where.kind === 'line' || problem.where.kind === 'lot') &&
        problem.where.lot === lotIndex,
    );

  // Which part of the screen the user is in decides which party the card shows.
  const [region, setRegion] = useState<Region | null>(null);
  const [editorCard, setEditorCard] = useState<number | null>(null);
  const focusedSales = salesRows.find((row) => row.rowId === salesFocus?.rowId);
  const focusedLot = lotRows.find((row) => row.rowId === lotsFocus?.rowId);
  const cardAccount =
    editorCard ??
    (region === 'sales'
      ? (focusedSales?.picked?.[SALE.buyer]?.id ?? null)
      : region === 'lots'
        ? (focusedLot?.picked?.[LOT.seller]?.id ?? null)
        : region === 'header'
          ? (zamindar?.id ?? null)
          : null);

  const rootRef = useRef<HTMLDivElement>(null);
  const focusTruck = () => {
    rootRef.current?.querySelector<HTMLInputElement>('input[role="combobox"]')?.focus();
  };
  // The user starts on the truck number, the first thing he types (PRD FR-12.2).
  useEffect(focusTruck, []);

  const totals = whole
    ? (preview?.summary ?? null)
    : (preview?.lots.find((lot) => lot.key === lotKey)?.totals ?? null);
  const lotPreview = preview?.lots.find((lot) => lot.key === lotKey);
  const netIsNegative = whole
    ? (preview?.lots.some((lot) => lot.net_is_negative) ?? false)
    : (lotPreview?.net_is_negative ?? false);

  const step = useLayoutStep();
  const wide = isFromStep(step, 'wide');
  const lotMoreForced = lotProblems.length > 0 || (lotRow !== undefined && isPukhtaSeller(lotRow));
  const card = <PartyCard accountId={cardAccount} />;

  // The cash note, the summary and the partner split: the numbers the munshi watches. They go to a
  // column of their own in a wide window, and under the sale lines in the others (F22).
  const numbers = (
    <>
      {preview === undefined ? null : <CashNote cash={preview.cash} />}
      {query.isError ? (
        <ErrorState
          title={<Localized id="bikri-preview-failed" />}
          error={uiErrorOf(query.error)}
          onRetry={() => {
            void query.refetch();
          }}
        />
      ) : preview !== undefined ? (
        <>
          <SummaryPanel
            totals={totals}
            whole={whole}
            lotNumber={lotIndex + 1}
            onWholeChange={(next) => {
              stores.view.getState().setWhole(next);
            }}
            netIsNegative={netIsNegative}
          />
          <SplitPanel preview={preview} />
        </>
      ) : query.isFetching ? (
        <LoadingState rows={6} />
      ) : (
        <EmptyState
          icon={FileText}
          title={<Localized id="bikri-empty-title" />}
          help={<Localized id="bikri-empty-help" />}
          action={
            <Button onClick={focusTruck}>
              <Localized id="bikri-empty-action" />
            </Button>
          }
        />
      )}
    </>
  );

  return (
    <div ref={rootRef} className="flex flex-col gap-3">
      <div className="flex flex-wrap items-baseline gap-x-4">
        {/* The side menu already says where we are, so the heading gives its room to the grids in
            the smallest window (design system 8, plan 12). Screen readers still hear it. */}
        <Localized id="glossary-bikri-book">
          <h1 className="sr-only normal:not-sr-only normal:type-title" />
        </Localized>
        <Localized id="bikri-help">
          <p className="type-body hidden text-ink-muted normal:block" />
        </Localized>
      </div>
      <div
        onFocusCapture={() => {
          setRegion('header');
        }}
      >
        <HeaderCard
          error={formError}
          onOpenBikri={onOpenBikri}
          onDone={() => {
            stores.lots.getState().setPendingFocus({ rowIndex: 0, field: LOT.seller });
          }}
        />
      </div>
      <ProblemList problems={problems} selectedLot={lotIndex} activeLine={activeLine} />
      <div className="grid grid-cols-1 gap-3 wide:grid-cols-3">
        <div className="flex min-w-0 flex-col gap-3 wide:col-span-2">
          <div
            className="flex flex-col gap-2"
            onFocusCapture={() => {
              setRegion('lots');
            }}
          >
            <LotsSection
              view={views.lots}
              onCardAccount={setEditorCard}
              moreForced={lotMoreForced}
            />
            {lotRow === undefined ? null : (
              <LotMore
                row={lotRow}
                lotIndex={lotIndex}
                lot={lotPreview}
                error={formError}
                hasProblem={lotProblems.length > 0}
              />
            )}
          </div>
          <div
            onFocusCapture={() => {
              setRegion('sales');
            }}
          >
            <SalesSection
              lotKey={lotKey}
              lotIndex={lotIndex}
              whole={whole}
              view={views.sales}
              onCardAccount={setEditorCard}
              card={wide ? null : card}
            />
          </div>
          {wide ? null : numbers}
          <ActiveLine
            lotKey={lotKey}
            lotIndex={lotIndex}
            preview={preview}
            error={formError}
            usesWeight={usesWeight}
          />
        </div>
        {wide ? (
          <div className="flex min-w-0 flex-col gap-3">
            {card}
            {numbers}
          </div>
        ) : null}
      </div>
      {/* The key numbers stay in view under the grids where the summary is far below (F22). */}
      {wide || preview === undefined ? null : (
        <NumbersStrip
          lotNumber={lotIndex + 1}
          lot={{
            totals: lotPreview?.totals ?? null,
            netIsNegative: lotPreview?.net_is_negative ?? false,
          }}
          whole={{
            totals: preview.summary,
            netIsNegative: preview.lots.some((lot) => lot.net_is_negative),
          }}
          cash={preview.cash}
        />
      )}
    </div>
  );
}

interface BikriBookProps extends EditorProps {
  /** Fills the draft before the screen opens (a saved draft in plan 13, the sample page now). */
  initialDraft?: ((stores: BikriDraftStores) => void) | undefined;
}

/** The screen with today known: the draft starts empty, with today as its date. */
function BikriBook({ today, onOpenBikri, initialDraft }: { today: string } & BikriBookProps) {
  const [stores] = useState(() => {
    const made = createBikriDraft(isoToDisplayDate(today));
    initialDraft?.(made);
    return made;
  });
  return (
    <BikriDraftProvider value={stores}>
      <Editor onOpenBikri={onOpenBikri} />
    </BikriDraftProvider>
  );
}

export function BikriBookPage({ initialDraft }: Pick<BikriBookProps, 'initialDraft'>) {
  const day = useDayStatus();
  if (day.isPending) return <LoadingState />;
  if (day.isError) {
    return (
      <ErrorState
        title={<Localized id="bikri-load-failed" />}
        error={uiErrorOf(day.error)}
        onRetry={() => {
          void day.refetch();
        }}
      />
    );
  }
  return <BikriBook today={day.data.today} initialDraft={initialDraft} />;
}
