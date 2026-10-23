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
import { ErrorState, LoadingState, showToast } from '../../../shared/ui';
import { vouchersPreview } from '../../vouchers';
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

/** The address of this screen. */
const KHATA_PATH = '/rokar/khata';

function KhataEditor({ today, urlType }: { today: string; urlType: KhataType }) {
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  const formRef = useFocusFirstField<HTMLFormElement>();
  const [draft, setDraft] = useState<KhataDraft>(() => newKhataDraft(urlType, today));
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
  const { query, current } = useEntryPreview(entry);
  const preview =
    entry === null || query.data?.type !== draft.type ? undefined : query.data.preview;
  const previewError = useMemo(() => previewRefusal(preview?.errors, tried), [preview, tried]);

  const draftText = useMemo(() => JSON.stringify(draft), [draft]);
  const saveKhata = useSaveKhata();
  const flow = useEntrySaveFlow({
    draftText,
    build: (confirm) => buildKhata(draft, confirm),
    save: (input) => saveKhata.mutateAsync(input),
    freshCash: (input) => vouchersPreview(input, null).then((answer) => answer.cash),
    cash: preview?.cash,
    current,
    onSaved: (saved) => {
      showToast(
        'success',
        l10n.getString('cashbook-saved', {
          kind: draft.type,
          number: l10n.getString(`vouchers-number-${kindId(draft.type)}`, {
            number: String(saved.number),
          }),
        }),
      );
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
      <KhataForm draft={draft} onChange={change} error={error} />
      <div className="flex flex-col gap-3">
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
      </div>
    </EntryFrame>
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
  return <KhataEditor key="new" today={day.data.today} urlType={search.type ?? 'khata_out'} />;
}
