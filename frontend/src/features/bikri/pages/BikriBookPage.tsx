// بکری بک (route /bikri/book; PRD 12.1 to 12.6, plans 12 and 13): the munshi types the whole بکری
// and sees every number live. One screen: the header, the lots, the sale lines of the selected lot,
// the summary (per lot and for the whole بکری), the live partner split, the cash note, and the
// party card of the party he is on. F10 saves it as one action (useSaveFlow), then a new empty
// بکری is ready. `/bikri/book?id=12` opens a saved بکری read-only, in the same layout.
//
// - The draft is kept safe: every 5 seconds the stores go to the backend (useDraftAutosave), and
//   after a crash the app offers it back (draft-offer). Leaving with typed work asks first
//   (LeaveGuard, PRD FR-7.9).
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
import { useNavigate, useSearch } from '@tanstack/react-router';
import { FileText } from 'lucide-react';
import { useEffect, useMemo, useRef, useState } from 'react';
import { LeaveGuard } from '../../../app/LeaveGuard';
import { useDayStatus } from '../../../app/use-day-status';
import type { BikriSaved } from '../../../bindings/BikriSaved';
import type { FieldError } from '../../../bindings/FieldError';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { draftsDiscard } from '../../../shared/drafts';
import { isoToDisplayDate, useGrouping } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { isFromStep, useLayoutStep } from '../../../shared/hooks';
import { SHORTCUTS, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  CashConfirmDialog,
  EmptyState,
  ErrorNotice,
  ErrorState,
  LinkButton,
  LoadingState,
  PartyCard,
  showToast,
  useErrorText,
} from '../../../shared/ui';
import { bikriFind } from '../api';
import { BikriToolbar } from '../components/BikriToolbar';
import { SavedBikriView } from '../components/saved/SavedBikriView';
import { PrintAction } from '../print/PrintAction';
import { PrintAfterSave } from '../print/PrintAfterSave';
import { usePendingRestore } from '../draft-offer';
import { BIKRI_DRAFT_SCREEN, applySnapshot } from '../draft-snapshot';
import { useBikriDetail } from '../hooks/useBikriReads';
import { useDraftAutosave } from '../hooks/useDraftAutosave';
import { useSaveFlow } from '../hooks/useSaveFlow';
import { afterSave } from '../after-save';
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
import { isFilled, isPukhtaSeller } from '../draft';
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

/** The problems of a refusal that point at a field, in the shape of the preview's problems. */
function refusalErrors(error: UiError | null): FieldError[] {
  if (error === null) return [];
  const items: FieldError[] = [];
  for (const item of error.errors) {
    if (item.field !== null && item.field !== '') {
      items.push({ code: item.code, field: item.field, values: item.values });
    }
  }
  return items;
}

/** The problems of the preview and of the last refusal, each once. */
function withoutRepeats(items: readonly FieldError[]): FieldError[] {
  const seen = new Set<string>();
  return items.filter((item) => {
    const key = `${item.code}|${item.field}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

interface EditorProps {
  /** A بکری was saved. The page opens a new one (and, in plan 14, prints for Ctrl+Enter). */
  onSaved: (saved: BikriSaved, andNext: boolean) => void;
  /** Opens a saved بکری by its id (read-only until plan 16). */
  onOpenSaved: (id: number) => void;
  /** The بکری saved last, with a way to open it. */
  lastSaved: { id: number; number: number } | null;
}

function Editor({ onSaved, onOpenSaved, lastSaved }: EditorProps) {
  const { l10n } = useLocalization();
  const grouping = useGrouping();
  const stores = useBikriDraft();
  const errorText = useErrorText();
  const language = useLanguage();
  const { query, stale, draft } = useBikriPreview(stores);
  const preview = query.data;
  useDraftFills(stores, preview, stale);

  // Safety and saving (plan 13). The draft goes to the backend every 5 seconds; a save stops that.
  const autosave = useDraftAutosave(stores, { enabled: true });
  const flow = useSaveFlow({
    stores,
    draft,
    preview,
    current: preview !== undefined && !stale,
    onSaved: (saved, andNext) => {
      autosave.stop();
      onSaved(saved, andNext);
    },
  });
  const [findOpen, setFindOpen] = useState(false);
  const dialogOpen = flow.ask !== null || findOpen;
  const dirty = isFilled(draft);
  useShortcut(SHORTCUTS.save, () => void flow.save(false), { enabled: !dialogOpen });
  useShortcut(SHORTCUTS.saveAndNext, () => void flow.save(true), { enabled: !dialogOpen });
  useShortcut(
    SHORTCUTS.find,
    () => {
      setFindOpen(true);
    },
    { enabled: !dialogOpen },
  );

  // The same-truck notice's "open" button: find the saved بکری by its number, then open it.
  const openByNumber = (number: number) => {
    void bikriFind({ number, truck_no: null, seller_id: null, date: null, item_id: null })
      .then((rows) => {
        const row = rows[0];
        if (row === undefined) throw new Error('not found');
        onOpenSaved(row.id);
      })
      .catch(() => {
        showToast(
          'error',
          <Localized id="bikri-open-failed">
            <span />
          </Localized>,
        );
      });
  };

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

  // The preview's problems, and what the backend refused on the last save. A problem that only
  // says "this is empty" waits until he tries to save (plan 12 contract).
  const refused = flow.refusal;
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
      <BikriToolbar
        currentId={null}
        onOpen={onOpenSaved}
        findOpen={findOpen}
        onFindOpenChange={setFindOpen}
        lastSaved={
          lastSaved === null ? null : (
            <span className="flex items-center gap-2 type-body text-ink-muted">
              <Localized id="bikri-last-saved" vars={{ number: String(lastSaved.number) }}>
                <span />
              </Localized>
              <LinkButton
                onClick={() => {
                  onOpenSaved(lastSaved.id);
                }}
              >
                <Localized id="bikri-truck-open" />
              </LinkButton>
            </span>
          )
        }
      >
        <PrintAction target={lastSaved} enabled={!dialogOpen} />
        <Button
          variant="primary"
          shortcut={SHORTCUTS.save}
          busy={flow.busy}
          onClick={() => {
            void flow.save(false);
          }}
        >
          <Localized id="bikri-save" />
        </Button>
      </BikriToolbar>
      {refused === null ? null : (
        <ErrorNotice
          error={refused}
          fieldsShown={refusalErrors(refused).map((item) => item.field)}
        />
      )}
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
          onOpenBikri={openByNumber}
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
      {flow.cashAfter === null ? null : (
        <CashConfirmDialog
          open={flow.ask !== null}
          onOpenChange={flow.onCashOpenChange}
          cashAfter={flow.cashAfter}
          onConfirm={flow.confirmCash}
          onCloseFocus={flow.onCashCloseFocus}
        />
      )}
      {/* Leaving with typed work asks first (PRD FR-7.9). "Leave" throws the draft away too. */}
      <LeaveGuard
        dirty={dirty}
        onSave={() => flow.save(false)}
        onLeave={() => {
          autosave.stop();
          void draftsDiscard(BIKRI_DRAFT_SCREEN).catch(() => null);
        }}
      />
    </div>
  );
}

interface BikriBookProps {
  /** Fills the draft before the screen opens (the sample page). */
  initialDraft?: ((stores: BikriDraftStores) => void) | undefined;
}

/**
 * A new draft: today as its date, or the draft the user chose to restore at start-up. The draft is
 * only read here (this runs again in development); the screen forgets it once it has opened.
 */
function newSession(today: string, initialDraft: BikriBookProps['initialDraft']) {
  const stores = createBikriDraft(isoToDisplayDate(today));
  const restored = usePendingRestore.getState().snapshot;
  if (restored !== null) applySnapshot(stores, restored);
  initialDraft?.(stores);
  return { stores, key: 0 };
}

/**
 * The new, unsaved بکری. After a save it opens another empty one (a new `key` remakes the editor,
 * so the first field has focus and nothing of the old one is left). The id of the بکری saved last
 * is kept here: it is the way back to it, and the hook that edit (plan 16) opens it with.
 */
function NewBikri({ today, initialDraft }: { today: string } & BikriBookProps) {
  const navigate = useNavigate();
  const [session, setSession] = useState(() => newSession(today, initialDraft));
  const [lastSaved, setLastSaved] = useState<{ id: number; number: number } | null>(null);
  const openSaved = (id: number) => {
    void navigate({ to: '/bikri/book', search: { id } });
  };
  // The screen has opened on the draft the user chose to restore: it is not waiting any more.
  useEffect(() => {
    usePendingRestore.getState().clear();
  }, []);

  return (
    <BikriDraftProvider value={session.stores}>
      <Editor
        key={session.key}
        lastSaved={lastSaved}
        onOpenSaved={openSaved}
        onSaved={(saved, andNext) => {
          setLastSaved({ id: saved.bikri_id, number: saved.number });
          afterSave(saved, andNext);
          setSession((current) => ({
            stores: createBikriDraft(isoToDisplayDate(today)),
            key: current.key + 1,
          }));
        }}
      />
    </BikriDraftProvider>
  );
}

/** A saved بکری opened read-only, with the same toolbar as the new one. */
function SavedBikri({ id }: { id: number }) {
  const navigate = useNavigate();
  const detail = useBikriDetail(id);
  const [findOpen, setFindOpen] = useState(false);
  useShortcut(
    SHORTCUTS.find,
    () => {
      setFindOpen(true);
    },
    { enabled: !findOpen },
  );
  const openSaved = (next: number) => {
    void navigate({ to: '/bikri/book', search: { id: next } });
  };
  const openNew = () => {
    void navigate({ to: '/bikri/book', search: {} });
  };

  let body;
  if (detail.isPending) {
    body = <LoadingState />;
  } else if (detail.isError) {
    body = (
      <ErrorState
        title={<Localized id="bikri-saved-failed" />}
        error={uiErrorOf(detail.error)}
        onRetry={() => {
          void detail.refetch();
        }}
      />
    );
  } else {
    body = <SavedBikriView detail={detail.data} />;
  }
  return (
    <div className="flex flex-col gap-3">
      <BikriToolbar
        currentId={id}
        onOpen={openSaved}
        findOpen={findOpen}
        onFindOpenChange={setFindOpen}
      >
        <PrintAction
          target={detail.data === undefined ? null : { id, number: detail.data.number }}
          enabled={!findOpen}
        />
        <Button variant="primary" shortcut={SHORTCUTS.newBikri} onClick={openNew}>
          <Localized id="bikri-new" />
        </Button>
      </BikriToolbar>
      {body}
    </div>
  );
}

/** The id in the address (`/bikri/book?id=12`), or null for a new بکری. */
function useSavedId(): number | null {
  const search = useSearch({ strict: false });
  const id: unknown = (search as { id?: unknown }).id;
  return typeof id === 'number' && Number.isSafeInteger(id) && id > 0 ? id : null;
}

function BikriBookContent({ initialDraft }: BikriBookProps) {
  const savedId = useSavedId();
  const day = useDayStatus();
  // A draft restored at start-up remakes the new بکری, even when it is already open.
  const restoreTick = usePendingRestore((state) => state.tick);
  if (savedId !== null) return <SavedBikri id={savedId} />;
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
  return <NewBikri key={restoreTick} today={day.data.today} initialDraft={initialDraft} />;
}

export function BikriBookPage({ initialDraft }: BikriBookProps) {
  return (
    <>
      {/* Ctrl+Enter saves and prints the documents of "print after save" (PRD FR-7.57, plan 14). */}
      <PrintAfterSave />
      <BikriBookContent initialDraft={initialDraft} />
    </>
  );
}
