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

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate, useSearch } from '@tanstack/react-router';
import { useMemo, useState } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import { uiErrorOf } from '../../../shared/api';
import { useEntrySaveFlow, useFocusFirstField } from '../../../shared/hooks';
import { ErrorState, LoadingState, showToast, FormAside } from '../../../shared/ui';
import {
  EntryBalances,
  EntryFrame,
  EntryNotes,
  kindId,
  previewRefusal,
  useEntryPreview,
  useFollowType,
} from '../../cashbook';
import { vouchersPreview } from '../../vouchers';
import {
  CAPITAL_FIELD_PATHS,
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
}

function CapitalEditor({ today, urlType, urlPartner }: EditorProps) {
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  const formRef = useFocusFirstField<HTMLFormElement>();
  const [draft, setDraft] = useState<CapitalDraft>(() =>
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
  const { query, current } = useEntryPreview(entry);
  // An answer about another type (kept on screen while the next one is on its way) is never shown.
  const preview =
    entry === null || query.data?.type !== draft.type ? undefined : query.data.preview;
  const previewError = useMemo(() => previewRefusal(preview?.errors, tried), [preview, tried]);

  const draftText = useMemo(() => JSON.stringify(draft), [draft]);
  const saveCapital = useSaveCapital();
  const flow = useEntrySaveFlow({
    draftText,
    build: (confirm) => buildCapital(draft, confirm),
    save: (input) => saveCapital.mutateAsync(input),
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
        form={<CapitalForm draft={draft} onChange={change} error={error} />}
        aside={
          <>
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

/** The route's page: it waits for today, then opens the form for the type in the address. */
export function CapitalPage() {
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
  return (
    <CapitalEditor
      key="new"
      today={day.data.today}
      urlType={search.type ?? 'capital_in'}
      urlPartner={search.partner}
    />
  );
}
