// The voucher screen (routes /bikri/voucher; PRD 15.1 to 15.4, FR-7.58 to FR-7.61, FR-7.67, FR-7.68,
// FR-6.7, FR-15.3): ONE screen with a type list at the top, وصولی, ادائیگی and پیشگی. Changing the
// type keeps the date, the amount, how the money moves and the narration, and clears the rest. The
// side menu items and Alt+V, Alt+A and Alt+P open it with the type chosen.
//
// - Keys: F10 or Ctrl+S saves, Ctrl+Enter saves, prints the slip if it is ticked and starts the
//   next, F2 puts the voucher in the waiting list and starts the next (while a saved voucher is
//   being changed it starts a new voucher, asking first if there is typed work), F9 prints, Esc
//   goes back.
// - Entries one after another (PRD FR-15.1): F2 puts the form in the waiting list and starts the
//   next; F10 saves the list, one voucher at a time, each its own posting event. One the backend
//   refuses stays in the list with its reason. Cash below zero asks per voucher (useSaveList).
//   A lone voucher is saved straight from the form, so a refusal stays at its fields.
// - A saved voucher opens for an edit while its day is open (`?id=`): the old posting is reversed
//   and the new one posted in one transaction. It can be cancelled with a reason.
// - Everything typed is text in this screen's state, so a language switch loses nothing (F39).
// - The frontend works out no money: the amount goes as typed; the balances, the bills settled, the
//   cash after and every warning come from the backend preview (the same function as the save).
// - Four states: loading while today (or a saved voucher) arrives, an error with a retry, the form,
//   and the list below it.

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { Ban, FileText, Plus, Printer } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { LeaveGuard } from '../../../app/LeaveGuard';
import type { UiError } from '../../../bindings/UiError';
import { SHORTCUTS, useEscape, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import { useFocusFirstField } from '../../../shared/hooks';
import {
  Button,
  CashConfirmDialog,
  ErrorNotice,
  Field,
  Notice,
  SaveChangesPrompt,
  SegmentedControl,
  showToast,
  type SegmentedOption,
} from '../../../shared/ui';
import { CancelVoucherDialog } from '../components/CancelVoucherDialog';
import { AdvanceHistory } from '../components/AdvanceHistory';
import { BillsPanel } from '../components/BillsPanel';
import { PayablesPanel } from '../components/PayablesPanel';
import { VoucherForm } from '../components/VoucherForm';
import { VoucherListSection } from '../components/VoucherListSection';
import { VoucherNotes } from '../components/VoucherNotes';
import { WaitingList } from '../components/WaitingList';
import {
  accountRequired,
  amountRequired,
  build,
  changeType,
  entryOf,
  helperTarget,
  isFilled,
  listed,
  newDraft,
  nextDraft,
  personSide,
  type ListedOutcome,
  type ListedVoucher,
} from '../draft';
import { useRowPreviews, useVoucherPreview } from '../hooks';
import { voucherNumberText } from '../list-columns';
import { VoucherPrintBox, type VoucherPrintTarget } from '../print/VoucherPrintBox';
import { usePrintVoucherAfterSave } from '../print/usePrintVoucherAfterSave';
import { SCREEN_TYPES, type ScreenType } from '../screen-type';
import { useSaveFlow } from '../useSaveFlow';
import { useSaveList } from '../useSaveList';
import type { VoucherDraft, VoucherRow, VoucherSaved } from '../types';

/** The fields the form shows next to their inputs, so the notice never tells a problem twice. */
const FIELD_PATHS = [
  'date',
  'account_id',
  'seller_account_id',
  'person_key',
  'amount',
  'collector_staff_id',
  'reason',
  'expected_goods',
  'narration',
];

const TYPE_OPTIONS: readonly SegmentedOption<ScreenType>[] = SCREEN_TYPES.map((type) => ({
  value: type,
  label: <Localized id={`vouchers-type-${type}`} />,
}));

interface VoucherEditorProps {
  /** Today, `YYYY-MM-DD`, from the day status. */
  today: string;
  /** The form to start with: empty for a new voucher, or the saved one for an edit. */
  initial: VoucherDraft;
  /** The saved voucher that is being edited, or null for a new one. */
  editing: VoucherRow | null;
  /** The type the address asks for (a new voucher follows it). */
  urlType: ScreenType;
}

export function VoucherEditor({ today, initial, editing, urlType }: VoucherEditorProps) {
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  const [draft, setDraft] = useState<VoucherDraft>(initial);
  const [editReason, setEditReason] = useState('');
  const [finished, setFinished] = useState(false);
  const [selected, setSelected] = useState<VoucherRow | null>(null);
  // The voucher F9 prints: the one that was last chosen in the list or last saved.
  const [printCandidate, setPrintCandidate] = useState<VoucherPrintTarget | null>(null);
  const [printTarget, setPrintTarget] = useState<VoucherPrintTarget | null>(null);
  const [cancelRow, setCancelRow] = useState<VoucherRow | null>(null);
  const [askNew, setAskNew] = useState(false);
  // The vouchers waiting to be saved together (PRD FR-15.1). Kept in this state, never in a grid,
  // so a language switch loses nothing (F39).
  const [list, setList] = useState<ListedVoucher[]>([]);
  const formRef = useFocusFirstField<HTMLFormElement>();
  const printAfterSave = usePrintVoucherAfterSave();
  const initialText = useMemo(() => JSON.stringify(initial), [initial]);

  const numberText = (type: ScreenType, number: number) =>
    l10n.getString(`vouchers-number-${type}`, { number: String(number) });

  // A new voucher follows the type in the address (the side menu, Alt+V, Alt+A, Alt+P). The draft
  // is adjusted while drawing, the way React advises for a value that follows a prop.
  const [seenType, setSeenType] = useState(urlType);
  if (seenType !== urlType) {
    setSeenType(urlType);
    if (editing === null) setDraft((current) => changeType(current, urlType));
  }

  const change = (patch: Partial<VoucherDraft>) => {
    setDraft((current) => ({ ...current, ...patch }));
  };

  const built = useMemo(() => build(draft, false), [draft]);
  const entry = useMemo(
    () => (built === null || draft.amount.trim() === '' ? null : entryOf(built)),
    [built, draft.amount],
  );
  const { query, current } = useVoucherPreview(entry, editing?.id ?? null);
  // The last answer stays on screen while the next one is on its way, so the notes do not flicker;
  // but an answer about another type of voucher is never shown (a receipt's bills under a payment).
  const preview =
    entry === null || query.data?.type !== entry.type ? undefined : query.data.preview;

  const afterSave = (saved: VoucherSaved, andNext: boolean) => {
    setPrintCandidate({
      id: saved.voucher_id,
      numberText: numberText(draft.type, saved.number),
    });
    if (andNext) printAfterSave(saved.voucher_id);
    if (editing !== null) {
      // The screen goes back to a new voucher once it has drawn as "nothing to save": the way out
      // is then not asked about (see the effect below).
      setFinished(true);
      return;
    }
    setDraft(nextDraft(draft));
    setEditReason('');
    // The next voucher starts at the account, the field that always changes.
    window.requestAnimationFrame(() => {
      formRef.current?.querySelector<HTMLInputElement>('input[role="combobox"]')?.focus();
    });
  };
  useEffect(() => {
    if (finished) void navigate({ to: '/bikri/voucher', search: { type: draft.type } });
  }, [finished, navigate, draft.type]);
  const flow = useSaveFlow({ draft, editing, editReason, preview, current, onSaved: afterSave });
  const listRun = useSaveList();
  const busy = flow.busy || listRun.busy;
  const rowEntries = useMemo(
    () =>
      list.map((voucher) => {
        const listedBuilt = build(voucher.draft, false);
        return {
          id: voucher.id,
          entry:
            listedBuilt === null || voucher.draft.amount.trim() === ''
              ? null
              : entryOf(listedBuilt),
        };
      }),
    [list],
  );
  const rowPreviews = useRowPreviews(rowEntries);

  // What the preview found, in the shape of a refused save, so the fields can show it.
  const previewError: UiError | null =
    preview === undefined || preview.errors.length === 0
      ? null
      : {
          ok: false,
          errors: preview.errors.map((item) => ({
            code: item.code,
            field: item.field,
            values: item.values,
          })),
          ref: '',
        };
  const error = flow.refusal ?? previewError;
  // An inactive account cannot be posted to (PRD FR-10.4, FR-7.38): the refusal says so at the
  // account field, and this note says what to do about it.
  const inactive = error?.errors.some((item) => item.code === 'ERR_ACCOUNT_INACTIVE') === true;

  const dirty =
    !finished &&
    (editing === null
      ? isFilled(draft) || list.length > 0
      : JSON.stringify(draft) !== initialText || editReason !== '');

  const focusAccount = () => {
    // The next voucher starts at the account, the field that always changes.
    window.requestAnimationFrame(() => {
      formRef.current?.querySelector<HTMLInputElement>('input[role="combobox"]')?.focus();
    });
  };

  // F2: the form goes into the waiting list and the next voucher starts (PRD FR-15.1).
  const addToList = () => {
    if (editing !== null || busy || !isFilled(draft)) return;
    if (draft.account === null) {
      flow.tell(accountRequired(draft));
      return;
    }
    if (draft.amount.trim() === '') {
      flow.tell(amountRequired());
      return;
    }
    // A problem the preview already found is at its field; the voucher is not listed with it.
    if (current && previewError !== null) return;
    setList((now) => [...now, listed(draft)]);
    setDraft(nextDraft(draft));
    focusAccount();
  };

  /** What one voucher of a run came to, put on the list: a saved one leaves it. */
  const applyOutcome = (outcome: ListedOutcome) => {
    setList((now) =>
      outcome.kind === 'saved'
        ? now.filter((voucher) => voucher.id !== outcome.id)
        : now.map((voucher) =>
            voucher.id === outcome.id
              ? {
                  ...voucher,
                  status:
                    outcome.kind === 'refused'
                      ? { kind: 'refused' as const, error: outcome.error }
                      : { kind: 'declined' as const },
                }
              : voucher,
          ),
    );
    if (outcome.kind === 'saved') {
      setPrintCandidate({
        id: outcome.saved.voucher_id,
        numberText: numberText(outcome.draft.type, outcome.saved.number),
      });
    }
  };

  // F10 with vouchers waiting: the form joins the list if it is filled, and every voucher is saved
  // in turn. True when all of them were saved.
  const saveAll = async (): Promise<boolean> => {
    if (busy) return false;
    let todo = list;
    if (isFilled(draft)) {
      if (draft.account === null) {
        flow.tell(accountRequired(draft));
        return false;
      }
      if (draft.amount.trim() === '') {
        flow.tell(amountRequired());
        return false;
      }
      if (current && previewError !== null) return false;
      todo = [...list, listed(draft)];
      setList(todo);
      setDraft(nextDraft(draft));
    }
    if (todo.length === 0) return true;
    const outcomes = await listRun.run(todo, applyOutcome);
    const savedOnes = outcomes.filter((outcome) => outcome.kind === 'saved');
    const refused = outcomes.filter((outcome) => outcome.kind === 'refused').length;
    const [only] = savedOnes;
    if (savedOnes.length === 1 && only?.kind === 'saved') {
      showToast(
        'success',
        l10n.getString('vouchers-saved', {
          kind: only.draft.type,
          number: numberText(only.draft.type, only.saved.number),
        }),
      );
    } else if (savedOnes.length > 1) {
      showToast('success', l10n.getString('vouchers-saved-many', { count: savedOnes.length }));
    }
    if (refused > 0)
      showToast('error', l10n.getString('vouchers-some-refused', { count: refused }));
    focusAccount();
    return outcomes.length === savedOnes.length;
  };

  // A lone voucher is saved from the form, so a refusal stays at its fields; with vouchers in the
  // list, the whole list is saved (a printed slip is for a lone voucher only).
  const saveNow = () => {
    if (list.length > 0) void saveAll();
    else void flow.save(false);
  };
  const saveAndNext = () => {
    if (list.length > 0) void saveAll();
    else void flow.save(true);
  };
  const navigation = useFormNavigation(saveNow);
  useShortcut(SHORTCUTS.save, saveNow);
  useShortcut(SHORTCUTS.saveAndNext, saveAndNext);

  const startNew = () => {
    setDraft(newDraft(draft.type, today));
    setEditReason('');
    if (editing !== null) void navigate({ to: '/bikri/voucher', search: { type: draft.type } });
  };
  const requestNew = () => {
    if (dirty) setAskNew(true);
    else startNew();
  };
  useShortcut(SHORTCUTS.newEntry, editing === null ? addToList : requestNew, { enabled: !askNew });
  useEscape('leave', () => {
    void navigate({ to: '/bikri' });
  });

  const requestPrint = () => {
    const target =
      editing !== null
        ? { id: editing.id, numberText: numberText(draft.type, editing.number) }
        : printCandidate;
    if (target === null) {
      showToast('error', l10n.getString('vouchers-print-save-first'));
      return;
    }
    setPrintTarget(target);
  };
  useShortcut(SHORTCUTS.print, requestPrint, { enabled: printTarget === null });

  const onType = (type: ScreenType) => {
    if (type === draft.type) return;
    setDraft((current) => changeType(current, type));
    void navigate({ to: '/bikri/voucher', search: { type }, replace: true });
  };
  // PRD FR-7.68: the note offered it; the user pressed the button. Seller, amount, date, how the
  // money moves and the narration stay, and the type becomes پیشگی.
  const makeAdvance = () => {
    setDraft((current) => ({
      ...changeType(current, 'advance'),
      account: current.account,
      accountType: current.accountType,
    }));
    void navigate({ to: '/bikri/voucher', search: { type: 'advance' }, replace: true });
  };

  const target = helperTarget(draft);
  const live = preview !== undefined && current ? preview : undefined;

  return (
    <div className="flex max-w-page flex-col gap-4">
      <LeaveGuard
        dirty={dirty}
        onSave={() => (list.length > 0 ? saveAll() : flow.save(false))}
        allowNavigation={(next) =>
          next.pathname === '/bikri/voucher' &&
          (next.search['id'] === undefined ? editing === null : next.search['id'] === editing?.id)
        }
      />
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex flex-col gap-1">
          <Localized id={`vouchers-${draft.type}-title`}>
            <h1 className="sr-only normal:not-sr-only normal:type-title" />
          </Localized>
          <Localized id={`vouchers-${draft.type}-help`}>
            <p className="hidden type-body text-ink-muted normal:block" />
          </Localized>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          {list.length === 0 ? null : (
            <Localized id="vouchers-waiting-count" vars={{ count: list.length }}>
              <span className="type-body text-ink-muted" />
            </Localized>
          )}
          {draft.type === 'receipt' && editing === null ? (
            <Button
              onClick={() => {
                void navigate({ to: '/bikri/collection' });
              }}
            >
              <FileText aria-hidden="true" className="size-icon-md" />
              <Localized id="vouchers-collection-open" />
            </Button>
          ) : null}
          <Button
            shortcut={SHORTCUTS.newEntry}
            disabled={editing === null && (busy || !isFilled(draft))}
            onClick={editing === null ? addToList : requestNew}
          >
            <Plus aria-hidden="true" className="size-icon-md" />
            <Localized id={editing === null ? 'vouchers-add-to-list' : 'vouchers-new'} />
          </Button>
          <Button shortcut={SHORTCUTS.print} onClick={requestPrint}>
            <Printer aria-hidden="true" className="size-icon-md" />
            <Localized id="vouchers-print" />
          </Button>
          <Button
            variant="primary"
            shortcut={SHORTCUTS.save}
            busy={busy}
            disabled={!dirty}
            onClick={saveNow}
          >
            <Localized id="layout-save-save" />
          </Button>
        </div>
      </div>
      <Field label={<Localized id="vouchers-field-type" />} hideLabel>
        {() => (
          <div>
            <SegmentedControl
              options={TYPE_OPTIONS}
              value={draft.type}
              groupLabel={l10n.getString('vouchers-field-type')}
              disabled={editing !== null}
              onChange={onType}
            />
          </div>
        )}
      </Field>
      {editing === null ? null : (
        <Notice
          tone="info"
          action={
            <Button
              variant="danger"
              onClick={() => {
                setCancelRow(editing);
              }}
            >
              <Ban aria-hidden="true" className="size-icon-md" />
              <Localized id="vouchers-cancel" />
            </Button>
          }
        >
          <Localized
            id="vouchers-editing-note"
            vars={{ kind: draft.type, number: numberText(draft.type, editing.number) }}
          />
        </Notice>
      )}
      <form
        ref={formRef}
        {...navigation}
        onSubmit={(event) => {
          event.preventDefault();
        }}
        className="flex flex-col gap-4"
      >
        {error === null ? null : <ErrorNotice error={error} fieldsShown={FIELD_PATHS} />}
        <div className="flex flex-col gap-4 rounded-3 border border-border bg-surface p-3">
          <VoucherForm
            draft={draft}
            onChange={change}
            error={error}
            editing={editing !== null}
            editReason={editReason}
            onEditReason={setEditReason}
            onMadeRegular={(made) => {
              // The account of the one-time person now exists: it takes his place in the form.
              change({
                account: {
                  id: made.account.id,
                  code: made.account.code,
                  name_ur: made.account.name_ur,
                  name_en: made.account.name_en,
                },
                accountType: personSide(draft),
                person: null,
                personText: '',
              });
            }}
          />
          {preview === undefined ? null : (
            <VoucherNotes draft={draft} preview={preview} onMakeAdvance={makeAdvance} />
          )}
          {inactive ? (
            <Notice tone="warn">
              <Localized id="vouchers-inactive-hint" />
            </Notice>
          ) : null}
        </div>
      </form>
      <WaitingList
        vouchers={list}
        previews={rowPreviews}
        disabled={busy}
        onRemove={(id) => {
          setList((now) => now.filter((voucher) => voucher.id !== id));
        }}
      />
      {target.kind === 'bills' ? (
        <BillsPanel
          accountId={target.accountId}
          personKey={target.personKey}
          effects={draft.type === 'receipt' ? (live?.bill_effects ?? null) : null}
        />
      ) : target.kind === 'payables' ? (
        <PayablesPanel sellerId={target.accountId} personKey={target.personKey} />
      ) : target.kind === 'advances' ? (
        <AdvanceHistory
          sellerId={target.accountId}
          effects={draft.type === 'receipt' ? (live?.advance_effects ?? null) : null}
        />
      ) : null}
      <VoucherListSection
        type={draft.type}
        today={today}
        selectedId={selected?.id ?? null}
        onSelect={(row) => {
          setSelected(row);
          if (row !== null) {
            setPrintCandidate({
              id: row.id,
              numberText: voucherNumberText(row, (id, vars) => l10n.getString(id, vars)),
            });
          }
        }}
        onEdit={(row) => {
          void navigate({ to: '/bikri/voucher', search: { type: draft.type, id: row.id } });
        }}
        onCancel={setCancelRow}
        onPrint={(row) => {
          setPrintTarget({
            id: row.id,
            numberText: voucherNumberText(row, (id, vars) => l10n.getString(id, vars)),
          });
        }}
      />
      {flow.cashAfter === null ? null : (
        <CashConfirmDialog
          open={flow.ask !== null}
          onOpenChange={(open) => {
            if (flow.onCashOpenChange(open)) {
              // After a "no" the cursor is on the amount (PRD FR-6.7).
              window.requestAnimationFrame(() => {
                formRef.current
                  ?.querySelector<HTMLInputElement>('input[inputmode="numeric"]')
                  ?.focus();
              });
            }
          }}
          cashAfter={flow.cashAfter}
          onConfirm={flow.confirmCash}
          onCloseFocus={(event) => {
            // Focus goes where the screen puts it (the amount after a no, the account after a
            // save), not back to the button.
            event.preventDefault();
          }}
        />
      )}
      {listRun.cashAfter === null ? null : (
        <CashConfirmDialog
          open={listRun.ask !== null}
          onOpenChange={(open) => {
            // After a "no" that voucher stays in the list and the run goes on with the rest.
            listRun.onCashOpenChange(open);
          }}
          cashAfter={listRun.cashAfter}
          onConfirm={listRun.confirmCash}
          onCloseFocus={(event) => {
            event.preventDefault();
          }}
        />
      )}
      <SaveChangesPrompt
        open={askNew}
        onOpenChange={setAskNew}
        onSave={() => {
          void flow.save(false).then((saved) => {
            if (saved) setAskNew(false);
          });
        }}
        onLeave={() => {
          setAskNew(false);
          startNew();
        }}
      />
      <CancelVoucherDialog
        row={cancelRow}
        onClose={() => {
          setCancelRow(null);
        }}
        onCancelled={() => {
          setCancelRow(null);
          if (editing !== null) setFinished(true);
        }}
      />
      {printTarget === null ? null : (
        <VoucherPrintBox
          open
          onOpenChange={(open) => {
            if (!open) setPrintTarget(null);
          }}
          target={printTarget}
        />
      )}
    </div>
  );
}
