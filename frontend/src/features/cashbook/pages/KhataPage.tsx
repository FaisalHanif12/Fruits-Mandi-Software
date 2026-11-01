// The الگ کھاتہ screen (route /rokar/khata; PRD 16.6, FR-16.6, FR-7.67, FR-6.7, E2E-38): ONE screen
// with a type list, رقم دی (paid out, the khata goes minus) and رقم آئی (received, it goes plus).
// `F4` in the khata field adds a new khata. A khata is never income or expense, is never split and
// is never in any partner's total. Below the form: the khata's entries in date order with the
// signed running balance.
//
// - The balance now and after, the cash note and the warnings come from the backend's preview (the
//   same function as the save, Code Standards B5). The frontend works out no money (CLAUDE.md
//   rule 1).
// - Paying out takes cash out, so it asks the cash question (PRD FR-6.7). Receiving never does.
// - Keys, the leave guard and the cash question come from EntryFrame. A language switch loses
//   nothing: the form is text in this screen's state (F39).
// - Four states: the day, the form, the preview (loading, error with a retry) and the list.

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate, useSearch } from '@tanstack/react-router';
import { useMemo, useState } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import { uiErrorOf } from '../../../shared/api';
import { useEntrySaveFlow, useFocusFirstField } from '../../../shared/hooks';
import { OpenOnItsScreen, finishFixSession } from '../../../shared/edit';
import { ErrorState, LoadingState, showToast, FormAside } from '../../../shared/ui';
import { useVoucherEditable, vouchersPreview } from '../../vouchers';
import {
  ClosedCashEntry,
  VoucherEditBoxes,
  VoucherEditTop,
  isReadyToEdit,
  useVoucherEdit,
} from '../edit-voucher';
import { khataDraftOf, useEditDraft } from '../edit-seed';
import { EntryBalances } from '../components/EntryBalances';
import { EntryFrame } from '../components/EntryFrame';
import { EntryNotes } from '../components/EntryNotes';
import { KhataForm } from '../components/KhataForm';
import { KhataListSection } from '../components/KhataListSection';
import { previewRefusal, useFollowType } from '../entry-support';
import { useEntryPreview, useSaveKhata } from '../hooks';
import {
  KHATA_FIELD_PATHS,
  buildKhata,
  changeKhataType,
  khataIsFilled,
  newKhataDraft,
  nextKhataDraft,
  type KhataDraft,
} from '../khata-draft';
import { kindId } from '../list-columns';
import { KHATA_TYPES, parseKhataSearch, type KhataType } from '../screen-type';
import type { VoucherEditable } from '../types';

/** The address of this screen. */
const KHATA_PATH = '/rokar/khata';

interface KhataEditorProps {
  today: string;
  urlType: KhataType;
  /** The saved voucher open for edit (plan 44); null for a new entry. */
  editable?: VoucherEditable | null;
  initial?: KhataDraft | null;
}

function KhataEditor({ today, urlType, editable = null, initial = null }: KhataEditorProps) {
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  const formRef = useFocusFirstField<HTMLFormElement>();
  const [draft, setDraft] = useState<KhataDraft>(() => initial ?? newKhataDraft(urlType, today));
  const [tried, setTried] = useState(false);
  useFollowType(urlType, (type) => {
    // The type list changed the draft already when it moved the address; only an address that
    // came from outside (Alt+K, a search result) changes it here.
    setDraft((current) => (current.type === type ? current : changeKhataType(current, type)));
    setTried(false);
  });

  const change = (patch: Partial<KhataDraft>) => {
    setDraft((current) => ({ ...current, ...patch }));
    setTried(false);
  };
  const filled = khataIsFilled(draft);

  const entry = useMemo(
    () => (draft.amount.trim() === '' ? null : buildKhata(draft, false)),
    [draft],
  );
  const edit = useVoucherEdit(editable, entry);
  const { query, current } = useEntryPreview(entry, edit?.editing.row.id ?? null);
  const preview =
    entry === null || query.data?.type !== draft.type ? undefined : query.data.preview;
  const previewError = useMemo(() => previewRefusal(preview?.errors, tried), [preview, tried]);

  const draftText = useMemo(() => JSON.stringify(draft), [draft]);
  const saveKhata = useSaveKhata();
  const flow = useEntrySaveFlow({
    draftText,
    build: (confirm) => buildKhata(draft, confirm),
    save: (input) => (edit === null ? saveKhata.mutateAsync(input) : edit.send(input)),
    freshCash: (input) =>
      vouchersPreview(input, edit?.editing.row.id ?? null).then((answer) => answer.cash),
    cash: preview?.cash,
    current,
    onSaved: (saved) => {
      const number = l10n.getString(`vouchers-number-${kindId(draft.type)}`, {
        number: String(saved.number),
      });
      if (edit !== null) {
        // An edit is done: the days reopened for it close again and a new entry starts (plan 44).
        showToast('success', l10n.getString('cashbook-edited', { number }));
        void finishFixSession(l10n);
        void navigate({ to: KHATA_PATH, search: { type: draft.type } });
        return;
      }
      showToast('success', l10n.getString('cashbook-saved', { kind: draft.type, number }));
      setDraft((previous) => nextKhataDraft(previous));
      setTried(false);
      window.requestAnimationFrame(() => {
        formRef.current?.querySelector<HTMLElement>('input')?.focus();
      });
    },
  });
  const error = flow.refusal ?? previewError;

  const save = async (): Promise<boolean> => {
    setTried(true);
    // An entry of an earlier day asks why it is changed, once (PRD FR-7.70).
    if (edit?.needsReason() === true) {
      edit.setAskReason(true);
      return false;
    }
    return flow.save();
  };
  const startNew = () => {
    setDraft((previous) => newKhataDraft(previous.type, today));
    setTried(false);
    window.requestAnimationFrame(() => {
      formRef.current?.querySelector<HTMLElement>('input')?.focus();
    });
  };
  const onType = (type: KhataType) => {
    if (type === draft.type) return;
    setDraft((previous) => changeKhataType(previous, type));
    setTried(false);
    void navigate({ to: KHATA_PATH, search: { type }, replace: true });
  };

  return (
    <EntryFrame<KhataType>
      titleId="cashbook-khata-screen-title"
      helpId="cashbook-khata-screen-help"
      typeLabelId="cashbook-khata-type"
      types={KHATA_TYPES.map((value) => ({ value, labelId: `cashbook-type-${kindId(value)}` }))}
      type={draft.type}
      onType={onType}
      top={edit === null ? undefined : <VoucherEditTop edit={edit} />}
      lockTypes={edit !== null}
      path={KHATA_PATH}
      filled={filled}
      busy={flow.busy}
      onSave={save}
      onNew={startNew}
      flow={flow}
      error={error}
      fieldsShown={KHATA_FIELD_PATHS}
      formRef={formRef}
      below={<KhataListSection khataId={draft.khata?.id ?? null} />}
    >
      <FormAside
        form={<KhataForm draft={draft} onChange={change} error={error} marks={edit?.marks} />}
        aside={
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
                  void navigate({ to: KHATA_PATH, search: { type: draft.type } });
                }}
              />
            )}
            <EntryBalances preview={preview} />
            <EntryNotes warnings={preview?.warnings ?? []} cash={preview?.cash} />
            {query.isError && entry !== null ? (
              <ErrorState
                title={<Localized id="cashbook-preview-failed" />}
                error={uiErrorOf(query.error)}
                onRetry={() => {
                  void query.refetch();
                }}
              />
            ) : null}
          </>
        }
      />
    </EntryFrame>
  );
}

/** A saved voucher open for edit (`?id=`): it waits for the voucher and its form, then edits it. */
function KhataEdit({ id, today }: { id: number; today: string }) {
  const editable = useVoucherEditable(id);
  const kind = editable.data?.entry.type;
  const isKhata = kind === undefined || KHATA_TYPES.some((type) => type === kind);
  const ready = editable.data !== undefined && isReadyToEdit(editable.data.row);
  const draft = useEditDraft(isKhata && ready ? editable.data : undefined, khataDraftOf);
  // An entry of another screen opens on its own screen.
  if (editable.data !== undefined && !isKhata) return <OpenOnItsScreen row={editable.data.row} />;
  if (editable.data !== undefined && !ready) {
    return <ClosedCashEntry titleId="cashbook-khata-screen-title" row={editable.data.row} />;
  }
  if (editable.isError || draft.isError) {
    return (
      <ErrorState
        title={<Localized id="cashbook-load-failed" />}
        error={uiErrorOf(editable.isError ? editable.error : draft.error)}
        onRetry={() => {
          void editable.refetch();
        }}
      />
    );
  }
  if (editable.data === undefined || draft.data === undefined) return <LoadingState />;
  return (
    <KhataEditor
      key={`${String(id)}-${String(editable.data.row.version)}`}
      today={today}
      urlType={draft.data.type}
      editable={editable.data}
      initial={draft.data}
    />
  );
}

/** The route's page: it waits for today, then opens the form for the type in the address. */
export function KhataPage() {
  const day = useDayStatus();
  const rawSearch: Record<string, unknown> = useSearch({ strict: false });
  const search = parseKhataSearch(rawSearch);
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
  if (search.id !== undefined) return <KhataEdit id={search.id} today={day.data.today} />;
  // A khata in the address (plan 49, PRD FR-7.79) starts the entry with it chosen; every amount stays
  // empty.
  const type = search.type ?? 'khata_out';
  return (
    <KhataEditor
      key={search.party === undefined ? 'new' : `new-${String(search.party.id)}`}
      today={day.data.today}
      urlType={type}
      initial={
        search.party === undefined
          ? null
          : { ...newKhataDraft(type, day.data.today), khata: search.party }
      }
    />
  );
}
