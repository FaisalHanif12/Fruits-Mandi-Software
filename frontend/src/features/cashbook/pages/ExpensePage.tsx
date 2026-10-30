// The خرچہ screen (route /rokar/expense; PRD 16.2, 16.3, FR-7.67, FR-7.68, FR-7.62, FR-16.8, FR-6.7,
// E2E-20, E2E-39): ONE screen with a type list at the top, دکان خرچہ, ذاتی خرچہ and ذاتی خرچہ واپسی.
// Alt+K opens it as دکان خرچہ. Changing the type keeps the date, the amount and the narration and
// clears the rest. Each type has its own number series, its own posting and its own checks.
//
// - Under the form the screen shows what the save will do, before the save (FR-16.8): each
//   partner's share of a shop expense, or the chosen partner's total now and after. The numbers
//   and their workings (ⓘ) come from the backend's preview, which is the same function as the save
//   (Code Standards B5). The frontend works out no money (CLAUDE.md rule 1).
// - The note of FR-7.68 ("is this a partner's own spending?") offers a button when "paid to" is a
//   partner's name in either script. The backend finds the match; the screen never switches by
//   itself.
// - Below: the day's shop expenses, or the partner's money taken (FR-16.2). "Repeat last month"
//   (FR-7.62) fills one regular expense at a time into the form.
// - Keys, the leave guard and the cash question come from EntryFrame. A language switch loses
//   nothing: the form is text in this screen's state (F39).
// - Four states: the day (loading, error), the form, the preview (loading, error with a retry) and
//   the lists below.
//
// Room for the پھل tab (PRD 16.7, plan 17d): `EXPENSE_TABS` in `expense-tabs.ts` is the one list
// of tabs of this area; the fruit screen adds its entry there and its route.

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate, useSearch } from '@tanstack/react-router';
import { CalendarClock } from 'lucide-react';
import { useMemo, useState } from 'react';
import type { FieldError } from '../../../bindings/FieldError';
import { useDayStatus } from '../../../app/use-day-status';
import { pickedOf } from '../../../shared/accounts';
import { uiErrorOf } from '../../../shared/api';
import { formatPaisa, isoToDisplayDate, useGrouping } from '../../../shared/format';
import { useEntrySaveFlow, useFocusFirstField } from '../../../shared/hooks';
import { displayName, useLanguage } from '../../../shared/i18n';
import { Button, ErrorState, FormAside, LoadingState, Notice, showToast } from '../../../shared/ui';
import { expensePreview } from '../api';
import { previewRefusal, useFollowType } from '../entry-support';
import { EntryFrame } from '../components/EntryFrame';
import { EntryNotes } from '../components/EntryNotes';
import { ExpenseForm } from '../components/ExpenseForm';
import { ExpenseListSection } from '../components/ExpenseListSection';
import { ExpensePreviewCard } from '../components/ExpensePreviewCard';
import { ExpenseTabs } from '../components/ExpenseTabs';
import { RepeatPanel } from '../components/RepeatPanel';
import {
  EXPENSE_FIELD_PATHS,
  buildExpense,
  changeExpenseType,
  chosenPartnerId,
  expenseIsFilled,
  newExpenseDraft,
  nextExpenseDraft,
  type ExpenseDraft,
} from '../expense-draft';
import { useExpensePreview, useSaveExpense } from '../hooks';
import { kindId } from '../list-columns';
import { EXPENSE_TYPES, parseExpenseSearch, type ExpenseType } from '../screen-type';
import type { RepeatedExpense } from '../types';

/** The address of this screen. */
const EXPENSE_PATH = '/rokar/expense';

const WHOLE_DATE = /^\d{1,2}-\d{1,2}-\d{4}$/;

interface ExpenseEditorProps {
  /** Today, `YYYY-MM-DD`, from the day status. */
  today: string;
  /** The type the address asks for. */
  urlType: ExpenseType;
}

function ExpenseEditor({ today, urlType }: ExpenseEditorProps) {
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  const language = useLanguage();
  const grouping = useGrouping();
  const formRef = useFocusFirstField<HTMLFormElement>();
  const [draft, setDraft] = useState<ExpenseDraft>(() => newExpenseDraft(urlType, today));
  const [tried, setTried] = useState(false);
  const [repeatOpen, setRepeatOpen] = useState(false);

  // The type in the address (Alt+K, the type list) is the type of the form.
  useFollowType(urlType, (type) => {
    // The type list changed the draft already when it moved the address; only an address that
    // came from outside (Alt+K, a search result) changes it here.
    setDraft((current) => (current.type === type ? current : changeExpenseType(current, type)));
    setTried(false);
  });

  const change = (patch: Partial<ExpenseDraft>) => {
    setDraft((current) => ({ ...current, ...patch }));
    setTried(false);
  };
  const filled = expenseIsFilled(draft);

  const entry = useMemo(
    () => (draft.amount.trim() === '' ? null : buildExpense(draft, false)),
    [draft],
  );
  const { query, current } = useExpensePreview(entry);
  // An answer about another type (kept on screen while the next one is on its way) is never shown.
  const preview =
    entry === null || query.data?.voucher_type !== draft.type ? undefined : query.data;
  const warnings: readonly FieldError[] = preview?.warnings ?? [];

  const previewError = useMemo(() => previewRefusal(preview?.errors, tried), [preview, tried]);

  const draftText = useMemo(() => JSON.stringify(draft), [draft]);
  const saveExpense = useSaveExpense();
  const flow = useEntrySaveFlow({
    draftText,
    build: (confirm) => buildExpense(draft, confirm),
    save: (input) => saveExpense.mutateAsync(input),
    freshCash: (input) => expensePreview(input, null).then((answer) => answer.cash),
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
      setDraft((previous) => nextExpenseDraft(previous));
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
    setDraft((previous) => newExpenseDraft(previous.type, today));
    setTried(false);
    window.requestAnimationFrame(() => {
      formRef.current?.querySelector<HTMLElement>('input')?.focus();
    });
  };

  const onType = (type: ExpenseType) => {
    if (type === draft.type) return;
    setDraft((previous) => changeExpenseType(previous, type));
    setTried(false);
    void navigate({ to: EXPENSE_PATH, search: { type }, replace: true });
  };
  // PRD FR-7.68: the note offered it; the user pressed the button. Date, amount and narration
  // stay, the partner is the one the backend found, and the type becomes ذاتی خرچہ.
  const makePersonal = (partnerId: number) => {
    setDraft((previous) => ({
      ...changeExpenseType(previous, 'personal_expense'),
      partnerId: String(partnerId),
    }));
    void navigate({ to: '/rokar/expense', search: { type: 'personal_expense' }, replace: true });
  };
  const fillFrom = (row: RepeatedExpense) => {
    setDraft((previous) => ({
      ...previous,
      head: pickedOf(row.head),
      amount: formatPaisa(row.amount_paisa, { grouping }),
      paidTo: row.paid_to,
      narration: row.narration,
    }));
    setTried(false);
    setRepeatOpen(false);
    window.requestAnimationFrame(() => {
      formRef.current?.querySelector<HTMLElement>('input')?.focus();
    });
  };

  const day = WHOLE_DATE.test(draft.date.trim()) ? draft.date.trim() : isoToDisplayDate(today);
  const partner = preview?.paid_to_partner ?? null;

  return (
    <EntryFrame<ExpenseType>
      titleId="cashbook-expense-title"
      helpId="cashbook-expense-help"
      tabs={<ExpenseTabs />}
      typeLabelId="cashbook-expense-type"
      types={EXPENSE_TYPES.map((value) => ({ value, labelId: `cashbook-type-${kindId(value)}` }))}
      type={draft.type}
      onType={onType}
      path={EXPENSE_PATH}
      filled={filled}
      busy={flow.busy}
      onSave={save}
      onNew={startNew}
      flow={flow}
      error={error}
      fieldsShown={EXPENSE_FIELD_PATHS[draft.type]}
      formRef={formRef}
      actions={
        draft.type === 'shop_expense' ? (
          <Button
            onClick={() => {
              setRepeatOpen((open) => !open);
            }}
          >
            <CalendarClock aria-hidden="true" className="size-icon-md" />
            <Localized id="cashbook-repeat-open" />
          </Button>
        ) : null
      }
      below={
        <>
          {repeatOpen && draft.type === 'shop_expense' ? (
            <RepeatPanel
              month={today.slice(0, 7)}
              onFill={fillFrom}
              onClose={() => {
                setRepeatOpen(false);
              }}
            />
          ) : null}
          <ExpenseListSection type={draft.type} day={day} partnerId={chosenPartnerId(draft)} />
        </>
      }
    >
      <FormAside
        form={<ExpenseForm draft={draft} onChange={change} error={error} />}
        aside={
          <>
            <ExpensePreviewCard type={draft.type} preview={preview} />
            <EntryNotes warnings={warnings} cash={preview?.cash}>
              {partner !== null && draft.type === 'shop_expense' ? (
                <Notice
                  tone="info"
                  action={
                    <Button
                      onClick={() => {
                        makePersonal(partner.id);
                      }}
                    >
                      <Localized id="cashbook-personal-make" />
                    </Button>
                  }
                >
                  <Localized
                    id="cashbook-personal-hint"
                    vars={{ name: displayName(partner, language) }}
                  />
                </Notice>
              ) : null}
            </EntryNotes>
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
export function ExpensePage() {
  const day = useDayStatus();
  const rawSearch: Record<string, unknown> = useSearch({ strict: false });
  const search = parseExpenseSearch(rawSearch);
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
  return <ExpenseEditor key="new" today={day.data.today} urlType={search.type ?? 'shop_expense'} />;
}
