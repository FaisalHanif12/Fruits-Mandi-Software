// The سرمایہ screen (route /partners/capital; PRD 17.4, FR-17.7, FR-17.8, FR-7.67, FR-6.7, E2E-35):
// ONE screen with a type list, سرمایہ جمع (a partner puts money in) and سرمایہ واپسی (he takes it
// back). Capital is never income or expense and is never split. Below the form: the partner's
// capital history with his balance now. Taking back more than the balance shows a warning and
// still saves (FR-17.8).
//
// - The balance now and after, the warning and the cash note come from the backend's preview (the
//   same function as the save, Code Standards B5). The frontend works out no money (CLAUDE.md
//   rule 1).
// - Taking capital back takes cash out, so it asks the cash question (PRD FR-6.7). Putting it in
//   never does.
// - Keys, the leave guard and the cash question come from EntryFrame. A language switch loses
//   nothing: the form is text in this screen's state (F39).
// - Four states: the day, the form, the preview (loading, error with a retry) and the list.

import { PartnerGate } from '../components/PartnerGate';
import { Localized, useLocalization } from '@fluent/react';
import { useNavigate, useSearch } from '@tanstack/react-router';
import { useMemo, useState } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import { uiErrorOf } from '../../../shared/api';
import { useEntrySaveFlow, useFocusFirstField } from '../../../shared/hooks';
import { OpenOnItsScreen, finishFixSession } from '../../../shared/edit';
import { ErrorState, LoadingState, showToast, FormAside } from '../../../shared/ui';
import {
  ClosedCashEntry,
  EntryBalances,
  EntryFrame,
  EntryNotes,
  VoucherEditBoxes,
  VoucherEditTop,
  isReadyToEdit,
  kindId,
  previewRefusal,
  useEditDraft,
  useEntryPreview,
  useFollowType,
  useVoucherEdit,
} from '../../cashbook';
import { useVoucherEditable, vouchersPreview, type VoucherEditable } from '../../vouchers';
import {
  CAPITAL_FIELD_PATHS,
  capitalDraftOf,
  buildCapital,
  capitalIsFilled,
  changeCapitalType,
  newCapitalDraft,
  nextCapitalDraft,
  type CapitalDraft,
} from '../capital-draft';
import { CapitalForm } from '../components/CapitalForm';
import { CapitalHistorySection } from '../components/CapitalHistorySection';
import { useSaveCapital } from '../money-hooks';
import { CAPITAL_TYPES, parseCapitalSearch, type CapitalType } from '../screen-type';

/** The address of this screen. */
const CAPITAL_PATH = '/partners/capital';

interface EditorProps {
  today: string;
  urlType: CapitalType;
  urlPartner: number | undefined;
  /** The saved voucher open for edit (plan 44); null for a new entry. */
  editable?: VoucherEditable | null;
  initial?: CapitalDraft | null;
}

function CapitalEditor({
  today,
  urlType,
  urlPartner,
  editable = null,
  initial = null,
}: EditorProps) {
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  const formRef = useFocusFirstField<HTMLFormElement>();
  const [draft, setDraft] = useState<CapitalDraft>(
    () =>
      initial ??
      newCapitalDraft(urlType, today, urlPartner === undefined ? '' : String(urlPartner)),
  );
  const [tried, setTried] = useState(false);
  useFollowType(urlType, (type) => {
    // The type list changed the draft already when it moved the address; only an address that
    // came from outside (a search result) changes it here.
    setDraft((current) => (current.type === type ? current : changeCapitalType(current, type)));
    setTried(false);
  });

  const change = (patch: Partial<CapitalDraft>) => {
    setDraft((current) => ({ ...current, ...patch }));
    setTried(false);
  };
  const filled = capitalIsFilled(draft);

  const entry = useMemo(
    () => (draft.amount.trim() === '' ? null : buildCapital(draft, false)),
    [draft],
  );
  const edit = useVoucherEdit(editable, entry);
  const { query, current } = useEntryPreview(entry, edit?.editing.row.id ?? null);
  // An answer about another type (kept on screen while the next one is on its way) is never shown.
  const preview =
    entry === null || query.data?.type !== draft.type ? undefined : query.data.preview;
  const previewError = useMemo(() => previewRefusal(preview?.errors, tried), [preview, tried]);

  const draftText = useMemo(() => JSON.stringify(draft), [draft]);
  const saveCapital = useSaveCapital();
  const flow = useEntrySaveFlow({
    draftText,
    build: (confirm) => buildCapital(draft, confirm),
    save: (input) => (edit === null ? saveCapital.mutateAsync(input) : edit.send(input)),
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
        void navigate({
          to: CAPITAL_PATH,
          search: {
            type: draft.type,
            ...(draft.partnerId === '' ? {} : { partner: Number(draft.partnerId) }),
          },
        });
        return;
      }
      showToast('success', l10n.getString('cashbook-saved', { kind: draft.type, number }));
      setDraft((previous) => nextCapitalDraft(previous));
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
    // The day, the type and the partner stay; what was typed is cleared.
    setDraft((previous) => nextCapitalDraft(previous));
    setTried(false);
    window.requestAnimationFrame(() => {
      formRef.current?.querySelector<HTMLElement>('input')?.focus();
    });
  };
  const onType = (type: CapitalType) => {
    if (type === draft.type) return;
    setDraft((previous) => changeCapitalType(previous, type));
    setTried(false);
    void navigate({
      to: CAPITAL_PATH,
      search: { type, ...(draft.partnerId === '' ? {} : { partner: Number(draft.partnerId) }) },
      replace: true,
    });
  };
  const partnerId = draft.partnerId === '' ? null : Number(draft.partnerId);

  return (
    <EntryFrame<CapitalType>
      titleId="partners-capital-screen-title"
      helpId="partners-capital-screen-help"
      typeLabelId="partners-capital-type"
      types={CAPITAL_TYPES.map((value) => ({ value, labelId: `cashbook-type-${kindId(value)}` }))}
      type={draft.type}
      onType={onType}
      top={edit === null ? undefined : <VoucherEditTop edit={edit} />}
      lockTypes={edit !== null}
      path={CAPITAL_PATH}
      leaveTo="/partners"
      filled={filled}
      busy={flow.busy}
      onSave={save}
      onNew={startNew}
      flow={flow}
      error={error}
      fieldsShown={CAPITAL_FIELD_PATHS}
      formRef={formRef}
      below={<CapitalHistorySection partnerId={partnerId} />}
    >
      <FormAside
        form={<CapitalForm draft={draft} onChange={change} error={error} marks={edit?.marks} />}
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
                  void navigate({ to: CAPITAL_PATH, search: { type: draft.type } });
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
function CapitalEdit({ id, today }: { id: number; today: string }) {
  const editable = useVoucherEditable(id);
  const kind = editable.data?.entry.type;
  const isCapital = kind === undefined || CAPITAL_TYPES.some((type) => type === kind);
  const ready = editable.data !== undefined && isReadyToEdit(editable.data.row);
  const draft = useEditDraft(isCapital && ready ? editable.data : undefined, capitalDraftOf);
  // An entry of another screen opens on its own screen.
  if (editable.data !== undefined && !isCapital) return <OpenOnItsScreen row={editable.data.row} />;
  if (editable.data !== undefined && !ready) {
    return <ClosedCashEntry titleId="partners-capital-screen-title" row={editable.data.row} />;
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
    <CapitalEditor
      key={`${String(id)}-${String(editable.data.row.version)}`}
      today={today}
      urlType={draft.data.type}
      urlPartner={undefined}
      editable={editable.data}
      initial={draft.data}
    />
  );
}

/** The route's page: it waits for today, then opens the form for the type in the address. */
function CapitalPageScreen() {
  const day = useDayStatus();
  const rawSearch: Record<string, unknown> = useSearch({ strict: false });
  const search = parseCapitalSearch(rawSearch);
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
  if (search.id !== undefined) return <CapitalEdit id={search.id} today={day.data.today} />;
  return (
    <CapitalEditor
      key="new"
      today={day.data.today}
      urlType={search.type ?? 'capital_in'}
      urlPartner={search.partner}
    />
  );
}

/** The screen behind the partner code (plan 44, PRD FR-17.11). */
export function CapitalPage() {
  return (
    <PartnerGate>
      <CapitalPageScreen />
    </PartnerGate>
  );
}
