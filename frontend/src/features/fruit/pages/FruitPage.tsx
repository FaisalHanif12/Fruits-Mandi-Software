// The پھل screen (route /rokar/fruit; PRD 16.7, FR-16.10 to FR-16.12, FR-7.53, FR-7.9, C27, E67): a
// tab next to خرچہ in the روکڑ area. Fruit a partner takes home, or fruit served to the shop's guests,
// recorded by item and quantity, from a lot of our shop or from another agent's shop.
//
// - Under the form the screen shows what the save will do, before the save: the value, each
//   partner's part with his total now and after, who is credited and the lot's balance. They come
//   from the backend's preview, the same function as the save (Code Standards B5). The frontend
//   works out no money (CLAUDE.md rule 1).
// - The rate is typed. The last rate of the item shows as grey text only (PRD FR-7.53).
// - Below: the list with its filters. An entry is edited (the form opens with its values and asks
//   why) or cancelled (a box asks why) while its day is open; both work by reversal (FR-16.12).
// - Keys, the leave guard and the new-entry prompt come from EntryFrame. A language switch loses
//   nothing: the form is text in this screen's state (F39).
// - Four states: the day (loading, error), the form, the preview (loading, error with a retry) and
//   the list below.

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate, useSearch } from '@tanstack/react-router';
import { useMemo, useState } from 'react';
import type { FieldError } from '../../../bindings/FieldError';
import { useDayStatus } from '../../../app/use-day-status';
import { uiErrorOf } from '../../../shared/api';
import { useGrouping } from '../../../shared/format';
import { useEntrySaveFlow, useFocusFirstField } from '../../../shared/hooks';
import { ErrorState, LoadingState, Notice, showToast, FormAside } from '../../../shared/ui';
import { EntryFrame, EntryNotes, ExpenseTabs, previewRefusal } from '../../cashbook';
import { CancelFruitDialog } from '../components/CancelFruitDialog';
import { FruitEntryView } from '../components/FruitEntryView';
import { FruitForm } from '../components/FruitForm';
import { FruitListSection } from '../components/FruitListSection';
import { FruitPreviewCard } from '../components/FruitPreviewCard';
import {
  FRUIT_FIELD_PATHS,
  buildFruit,
  buildSaveRequest,
  draftFromRow,
  fruitIsFilled,
  newFruitDraft,
  nextFruitDraft,
  readyToAsk,
} from '../draft';
import { useFruitEntry, useFruitPreview, useReadFruit, useSaveFruit } from '../hooks';
import { fruitNumberText } from '../list-columns';
import { parseFruitSearch, type FruitSearch } from '../screen-search';
import { FOR_GUESTS, type EditingFruit, type FruitDraft, type FruitRow } from '../types';

/** The address of this screen: a tab of the خرچہ area (expense-tabs.ts). */
const FRUIT_PATH = '/rokar/fruit';

interface FruitEditorProps {
  /** Today, `YYYY-MM-DD`, from the day status. */
  today: string;
  /** The entry the address opens in its saved view (`?entry=7`), a bad id, or nothing. */
  entry: FruitSearch['entry'];
}

function FruitEditor({ today, entry: openEntry }: FruitEditorProps) {
  const navigate = useNavigate();
  const { l10n } = useLocalization();
  const grouping = useGrouping();
  const formRef = useFocusFirstField<HTMLFormElement>();
  const [draft, setDraft] = useState<FruitDraft>(() => newFruitDraft(today));
  const [editing, setEditing] = useState<EditingFruit | null>(null);
  const [tried, setTried] = useState(false);
  const [selectedId, setSelectedId] = useState<number | null>(
    typeof openEntry === 'number' ? openEntry : null,
  );
  // The address can open another entry while the screen is up: the list follows it (the form is not
  // touched, so nothing typed is lost).
  const [seenEntry, setSeenEntry] = useState(openEntry);
  if (seenEntry !== openEntry) {
    setSeenEntry(openEntry);
    if (typeof openEntry === 'number') setSelectedId(openEntry);
  }
  const opened = useFruitEntry(typeof openEntry === 'number' ? openEntry : null);
  const [cancelRow, setCancelRow] = useState<FruitRow | null>(null);
  const read = useReadFruit();
  const say = (id: string, vars?: Record<string, string>) => l10n.getString(id, vars);

  const change = (update: (current: FruitDraft) => FruitDraft) => {
    setDraft(update);
    setTried(false);
  };
  const filled = fruitIsFilled(draft);

  // The backend is asked as soon as the item is known, so the last rate can show as a hint before
  // the rate is typed (PRD FR-7.53); what is still empty is told only when the user tries to save.
  const entry = useMemo(
    () => (readyToAsk(draft) ? buildFruit(draft, editing === null ? null : editing.id) : null),
    [draft, editing],
  );
  const { query, current } = useFruitPreview(entry);
  const preview = entry === null ? undefined : query.data;
  const warnings: readonly FieldError[] = preview?.warnings ?? [];
  const previewError = useMemo(() => previewRefusal(preview?.errors, tried), [preview, tried]);

  const draftText = useMemo(() => JSON.stringify(draft), [draft]);
  const saveFruit = useSaveFruit();
  const flow = useEntrySaveFlow({
    draftText,
    build: () => buildSaveRequest(draft, editing),
    save: (request) => saveFruit.mutateAsync(request),
    // No cash moves on this screen, so the cash question never comes.
    freshCash: null,
    cash: undefined,
    current,
    onSaved: (saved) => {
      showToast(
        'success',
        l10n.getString('fruit-saved', { number: fruitNumberText(saved.number, say) }),
      );
      setDraft((previous) => nextFruitDraft(previous));
      setEditing(null);
      setTried(false);
      window.requestAnimationFrame(() => {
        formRef.current?.querySelector<HTMLElement>('input, button')?.focus();
      });
    },
  });
  const error = flow.refusal ?? previewError;

  const save = async (): Promise<boolean> => {
    setTried(true);
    return flow.save();
  };
  const startNew = () => {
    setDraft((previous) => ({ ...newFruitDraft(''), date: previous.date, forWhom: '' }));
    setEditing(null);
    setTried(false);
    window.requestAnimationFrame(() => {
      formRef.current?.querySelector<HTMLElement>('input, button')?.focus();
    });
  };
  // PRD FR-16.12: the form opens with the entry as the database has it now, and asks why.
  const startEdit = (row: FruitRow) => {
    read.mutate(row.id, {
      onSuccess: (fresh) => {
        setDraft(draftFromRow(fresh, grouping));
        setEditing({ id: fresh.id, version: fresh.version, number: fresh.number });
        setTried(false);
        window.requestAnimationFrame(() => {
          formRef.current?.querySelector<HTMLElement>('input, button')?.focus();
        });
      },
    });
  };

  return (
    <div className="flex max-w-page flex-col gap-3 short:gap-2">
      {openEntry === undefined ? null : (
        <FruitEntryView
          entry={openEntry}
          onEdit={startEdit}
          onCancel={setCancelRow}
          onClose={() => {
            void navigate({ to: FRUIT_PATH, search: {}, replace: true });
          }}
        />
      )}
      <EntryFrame<string>
        titleId="fruit-title"
        helpId="fruit-help"
        tabs={<ExpenseTabs />}
        path={FRUIT_PATH}
        filled={filled}
        busy={flow.busy}
        onSave={save}
        onNew={startNew}
        flow={flow}
        error={error}
        fieldsShown={FRUIT_FIELD_PATHS}
        formRef={formRef}
        below={
          <>
            <FruitListSection
              key={opened.data?.entry_date ?? today}
              today={today}
              day={opened.data?.entry_date}
              selectedId={selectedId}
              onSelect={(row) => {
                setSelectedId(row === null ? null : row.id);
              }}
              onEdit={startEdit}
              onCancel={setCancelRow}
            />
            <CancelFruitDialog
              row={cancelRow}
              onClose={() => {
                setCancelRow(null);
              }}
            />
          </>
        }
      >
        {editing === null ? null : (
          <Notice tone="info">
            <Localized id="fruit-editing" vars={{ number: fruitNumberText(editing.number, say) }} />
          </Notice>
        )}
        {read.isError ? (
          <ErrorState
            title={<Localized id="fruit-load-failed" />}
            error={uiErrorOf(read.error)}
            onRetry={() => {
              read.reset();
            }}
          />
        ) : null}
        <FormAside
          form={
            <FruitForm
              draft={draft}
              onChange={change}
              error={error}
              preview={preview}
              editing={editing !== null}
            />
          }
          aside={
            <>
              <FruitPreviewCard preview={preview} guests={draft.forWhom === FOR_GUESTS} />
              <EntryNotes warnings={warnings} cash={undefined} />
              {query.isError && entry !== null ? (
                <ErrorState
                  title={<Localized id="fruit-numbers-failed" />}
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
    </div>
  );
}

/** The route's page: it waits for today, then opens the form. */
export function FruitPage() {
  const day = useDayStatus();
  const rawSearch: Record<string, unknown> = useSearch({ strict: false });
  const search = parseFruitSearch(rawSearch);
  if (day.isPending) return <LoadingState />;
  if (day.isError) {
    return (
      <ErrorState
        title={<Localized id="fruit-load-failed" />}
        error={uiErrorOf(day.error)}
        onRetry={() => {
          void day.refetch();
        }}
      />
    );
  }
  return <FruitEditor key="new" today={day.data.today} entry={search.entry} />;
}
