// The ادھار رقم screen (route /rokar/lent; PRD 16.5, FR-16.4, FR-16.5, FR-16.9, FR-7.67, FR-6.7,
// E2E-30): ONE screen with a type list, money lent (رقم دی) and money returned (واپسی). A return is
// in cash or deducted from the person's salary (the one place a "mode" exists in this area). `F4`
// in the person field adds a new person. Below the form: the person's loans, with what was
// returned and what is still open.
//
// - The balance now and after, the cash note and the warnings come from the backend's preview (the
//   same function as the save, Code Standards B5). The frontend works out no money (CLAUDE.md
//   rule 1).
// - Money lent takes cash out, so it asks the cash question (PRD FR-6.7). A return never does.
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
import { vouchersPreview } from '../../vouchers';
import { previewRefusal, useFollowType } from '../entry-support';
import { EntryBalances } from '../components/EntryBalances';
import { EntryFrame } from '../components/EntryFrame';
import { EntryNotes } from '../components/EntryNotes';
import { LentForm } from '../components/LentForm';
import { LentListSection } from '../components/LentListSection';
import { useEntryPreview, useSaveLent } from '../hooks';
import {
  LENT_FIELD_PATHS,
  buildLent,
  changeLentType,
  lentIsFilled,
  newLentDraft,
  nextLentDraft,
  type LentDraft,
} from '../lent-draft';
import { kindId } from '../list-columns';
import { LENT_TYPES, parseLentSearch, type LentType } from '../screen-type';

/** The address of this screen. */
const LENT_PATH = '/rokar/lent';

function LentEditor({ today, urlType }: { today: string; urlType: LentType }) {
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  const formRef = useFocusFirstField<HTMLFormElement>();
  const [draft, setDraft] = useState<LentDraft>(() => newLentDraft(urlType, today));
  const [tried, setTried] = useState(false);
  useFollowType(urlType, (type) => {
    // The type list changed the draft already when it moved the address; only an address that
    // came from outside (Alt+K, a search result) changes it here.
    setDraft((current) => (current.type === type ? current : changeLentType(current, type)));
    setTried(false);
  });

  const change = (patch: Partial<LentDraft>) => {
    setDraft((current) => ({ ...current, ...patch }));
    setTried(false);
  };
  const filled = lentIsFilled(draft);

  const entry = useMemo(
    () => (draft.amount.trim() === '' ? null : buildLent(draft, false)),
    [draft],
  );
  const { query, current } = useEntryPreview(entry);
  // An answer about another type (kept on screen while the next one is on its way) is never shown.
  const preview =
    entry === null || query.data?.type !== draft.type ? undefined : query.data.preview;
  const previewError = useMemo(() => previewRefusal(preview?.errors, tried), [preview, tried]);

  const draftText = useMemo(() => JSON.stringify(draft), [draft]);
  const saveLent = useSaveLent();
  const flow = useEntrySaveFlow({
    draftText,
    build: (confirm) => buildLent(draft, confirm),
    save: (input) => saveLent.mutateAsync(input),
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
      setDraft((previous) => nextLentDraft(previous));
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
    setDraft((previous) => newLentDraft(previous.type, today));
    setTried(false);
    window.requestAnimationFrame(() => {
      formRef.current?.querySelector<HTMLElement>('input')?.focus();
    });
  };
  const onType = (type: LentType) => {
    if (type === draft.type) return;
    setDraft((previous) => changeLentType(previous, type));
    setTried(false);
    void navigate({ to: LENT_PATH, search: { type }, replace: true });
  };

  return (
    <EntryFrame<LentType>
      titleId="cashbook-lent-screen-title"
      helpId="cashbook-lent-screen-help"
      typeLabelId="cashbook-lent-type"
      types={LENT_TYPES.map((value) => ({ value, labelId: `cashbook-type-${kindId(value)}` }))}
      type={draft.type}
      onType={onType}
      path={LENT_PATH}
      filled={filled}
      busy={flow.busy}
      onSave={save}
      onNew={startNew}
      flow={flow}
      error={error}
      fieldsShown={LENT_FIELD_PATHS[draft.type]}
      formRef={formRef}
      below={<LentListSection personId={draft.person?.id ?? null} />}
    >
      <FormAside
        form={<LentForm draft={draft} onChange={change} error={error} />}
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
export function LentPage() {
  const day = useDayStatus();
  const rawSearch: Record<string, unknown> = useSearch({ strict: false });
  const search = parseLentSearch(rawSearch);
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
  return <LentEditor key="new" today={day.data.today} urlType={search.type ?? 'money_lent'} />;
}
