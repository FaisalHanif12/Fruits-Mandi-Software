// The entry form of one garden (PRD FR-7.67, FR-12.35, FR-12.36): a type list at the top, قسط,
// باغ خرچ and دوسری دکان میں فروخت, and under it only the fields of that type, always in the same
// places: the day first, then the fields of the type, then where each partner's money comes from.
// Changing the type keeps the day, the amount and the note, and clears the rest. Each type has its
// own number series, its own posting and its own checks.
//
// - Keys: F10 or Ctrl+S saves, F2 starts a new entry (asking first if there is typed work). The
//   form gets its Enter and arrow keys from shared/keyboard (F27).
// - Everything typed is text in this screen's state, so a language switch loses nothing (F39).
// - The frontend works out no money: every amount, each partner's part, the running balance after
//   the entry, the cash after it and every warning come from the backend preview, which is the
//   same function as the save (Code Standards B5).
// - The sources default to the garden's and can be changed on the entry (PRD A54). A shop part
//   comes off the partner's total and out of روکڑ; a pocket part is only recorded (FR-12.30), and
//   the form says so in words.
// - Four states: the form (empty), the preview loading or failed (an error with a retry), and the
//   numbers. The garden's own loading and error states are on the page.

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { Plus, TriangleAlert } from 'lucide-react';
import { useMemo, useRef, useState } from 'react';
import { LeaveGuard } from '../../../app/LeaveGuard';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { useEntrySaveFlow, useFocusFirstField, useRefusalIntoView } from '../../../shared/hooks';
import { SHORTCUTS, useEscape, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  CashConfirmDialog,
  CashNote,
  DateInput,
  EntryPartsCard,
  EntrySources,
  ErrorNotice,
  ErrorState,
  Field,
  Notice,
  SaveChangesPrompt,
  SegmentedControl,
  fieldError,
  showToast,
  useErrorText,
} from '../../../shared/ui';
import { gardensEntryPreview } from '../api';
import {
  ENTRY_FIELD_PATHS,
  buildArrival,
  buildEntry,
  changeType,
  entryIsFilled,
  newEntryDraft,
  nextEntryDraft,
} from '../draft';
import { useGardenEntryPreview, useSaveGardenArrival, useSaveGardenEntry } from '../hooks';
import { arrivalNumberText } from '../list-columns';
import {
  ENTRY_TYPES,
  type EntryDraft,
  type EntryType,
  type GardenEntryInput,
  type GardenEntrySaved,
  type GardenRow,
} from '../types';
import { GardenEntryFieldsForm } from './GardenEntryFieldsForm';
import { GardenEntryNumbersCard } from './GardenEntryNumbersCard';

/** Codes that only say a field is empty. Held back until the user tries to save. */
const INCOMPLETE: ReadonlySet<string> = new Set(['ERR_REQUIRED', 'ERR_REASON_REQUIRED']);

interface GardenEntryEditorProps {
  garden: GardenRow;
  /** Today, `YYYY-MM-DD`. */
  today: string;
}

export function GardenEntryEditor({ garden, today }: GardenEntryEditorProps) {
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  const errorText = useErrorText();
  const [draft, setDraft] = useState<EntryDraft>(() => newEntryDraft('installment', today));
  const [tried, setTried] = useState(false);
  // Goods arrived have no preview and no cash: they are saved by their own command, and what the
  // backend refuses is kept here.
  const [arrivalRefusal, setArrivalRefusal] = useState<UiError | null>(null);
  const [askNew, setAskNew] = useState(false);
  const formRef = useFocusFirstField<HTMLFormElement>();
  const rootRef = useRef<HTMLDivElement>(null);

  const change = (patch: Partial<EntryDraft>) => {
    setDraft((current) => ({ ...current, ...patch }));
    setTried(false);
    setArrivalRefusal(null);
  };
  const filled = entryIsFilled(draft);

  // Nothing is asked for an empty form; the preview is one function for every type.
  const arrival = draft.type === 'arrival';
  const input: GardenEntryInput | null = useMemo(
    () => (filled && !arrival ? buildEntry(draft, garden.id, garden.partners, false) : null),
    [arrival, draft, filled, garden.id, garden.partners],
  );
  const view = useGardenEntryPreview(input);
  const preview = input === null ? undefined : view.query.data;
  const previewFailed = input !== null && view.query.isError;
  const payment = draft.type === 'installment' || draft.type === 'cost';

  // What the preview found, in the shape of a refused save, so the fields can show it. Problems
  // that only say "this is empty" wait until the user tries to save, so a field does not turn red
  // before he has reached it.
  const previewError: UiError | null = useMemo(() => {
    const shown = (preview?.errors ?? []).filter((item) => tried || !INCOMPLETE.has(item.code));
    return shown.length === 0
      ? null
      : {
          ok: false,
          errors: shown.map((item) => ({
            code: item.code,
            field: item.field,
            values: item.values,
          })),
          ref: '',
        };
  }, [preview?.errors, tried]);

  const draftText = useMemo(() => JSON.stringify(draft), [draft]);
  const afterSave = () => {
    setDraft((previous) => nextEntryDraft(previous));
    setTried(false);
    setArrivalRefusal(null);
    window.requestAnimationFrame(() => {
      formRef.current?.querySelector<HTMLElement>('input')?.focus();
    });
  };

  const saveEntry = useSaveGardenEntry();
  const saveArrival = useSaveGardenArrival();
  const flow = useEntrySaveFlow<GardenEntryInput, GardenEntrySaved>({
    draftText,
    build: (confirm) =>
      buildEntry(
        // The flow is only used for a money entry; goods arrived never reach it.
        arrival ? newEntryDraft('installment', today) : draft,
        garden.id,
        garden.partners,
        confirm,
      ),
    save: (built) => saveEntry.mutateAsync(built),
    freshCash: (built) => gardensEntryPreview(built).then((answer) => answer.cash),
    cash: preview?.cash,
    current: view.current,
    onSaved: (saved) => {
      showToast(
        'success',
        l10n.getString(`gardens-saved-${draft.type.replaceAll('_', '-')}`, {
          number: String(saved.number),
        }),
      );
      afterSave();
    },
  });

  const error = arrival ? arrivalRefusal : (flow.refusal ?? previewError);
  useRefusalIntoView(rootRef, arrival ? arrivalRefusal : flow.refusal);

  // The weight field shows for an item sold by weight (the backend asks for it) or once typed.
  const showWeight =
    draft.weight.trim() !== '' ||
    arrivalRefusal?.errors.some((item) => item.code === 'ERR_WEIGHT_REQUIRED') === true;

  const save = async (): Promise<boolean> => {
    setTried(true);
    if (!arrival) return flow.save();
    setArrivalRefusal(null);
    try {
      const saved = await saveArrival.mutateAsync(buildArrival(draft, garden.id));
      showToast(
        'success',
        l10n.getString('gardens-saved-arrival', {
          number: arrivalNumberText(saved.row.number, (id, vars) => l10n.getString(id, vars)),
        }),
      );
      afterSave();
      return true;
    } catch (failure: unknown) {
      setArrivalRefusal(uiErrorOf(failure));
      return false;
    }
  };
  const saveNow = () => {
    void save();
  };
  const navigation = useFormNavigation(saveNow);
  useShortcut(SHORTCUTS.save, saveNow);

  const startNew = () => {
    setDraft((previous) => newEntryDraft(previous.type, today));
    setTried(false);
    window.requestAnimationFrame(() => {
      formRef.current?.querySelector<HTMLElement>('input')?.focus();
    });
  };
  const requestNew = () => {
    if (filled) setAskNew(true);
    else startNew();
  };
  useShortcut(SHORTCUTS.newEntry, requestNew, { enabled: !askNew });
  useEscape('leave', () => {
    void navigate({ to: '/gardens' });
  });

  const typeOptions = ENTRY_TYPES.map((type) => ({
    value: type,
    label: l10n.getString(`gardens-type-${type.replaceAll('_', '-')}`),
  }));
  const warnings = preview?.warnings ?? [];
  const parts = preview?.numbers?.parts ?? null;

  return (
    <div ref={rootRef} className="flex flex-col gap-3">
      <LeaveGuard dirty={filled} onSave={save} />
      <div className="flex flex-wrap items-center justify-between gap-3">
        <SegmentedControl<EntryType>
          groupLabel={l10n.getString('gardens-type-label')}
          options={typeOptions}
          value={draft.type}
          onChange={(type) => {
            setDraft((current) => changeType(current, type));
            setTried(false);
            setArrivalRefusal(null);
          }}
        />
        <div className="flex flex-wrap items-center gap-2">
          <Button shortcut={SHORTCUTS.newEntry} onClick={requestNew}>
            <Plus aria-hidden="true" className="size-icon-md" />
            <Localized id="gardens-entry-new" />
          </Button>
          <Button
            variant="primary"
            shortcut={SHORTCUTS.save}
            busy={arrival ? saveArrival.isPending : flow.busy}
            disabled={!filled}
            onClick={saveNow}
          >
            <Localized id="layout-save-save" />
          </Button>
        </div>
      </div>
      <form
        ref={formRef}
        {...navigation}
        onSubmit={(event) => {
          event.preventDefault();
        }}
        className="flex flex-col gap-4"
      >
        {error === null ? null : (
          <ErrorNotice error={error} fieldsShown={ENTRY_FIELD_PATHS[draft.type]} />
        )}
        <div className="grid grid-cols-1 gap-4 normal:grid-cols-5">
          <div className="grid grid-cols-2 gap-4 rounded-3 border border-border bg-surface p-3 normal:col-span-3">
            <Field label={<Localized id="gardens-field-date" />} error={fieldError(error, 'date')}>
              {(props) => (
                <DateInput
                  {...props}
                  value={draft.date}
                  onChange={(date) => {
                    change({ date });
                  }}
                />
              )}
            </Field>
            <div />
            <GardenEntryFieldsForm
              draft={draft}
              onChange={change}
              error={error}
              showWeight={showWeight}
            />
            {payment ? (
              <div className="col-span-2 flex flex-col gap-1">
                <Localized id="gardens-sources-title">
                  <h3 className="type-label text-ink-muted" />
                </Localized>
                <Localized id="gardens-sources-hint">
                  <p className="type-body text-ink-muted" />
                </Localized>
                <EntrySources
                  partners={garden.partners}
                  value={draft.sources}
                  onChange={(partnerId, source) => {
                    change({ sources: { ...draft.sources, [partnerId]: source } });
                  }}
                />
              </div>
            ) : null}
          </div>
          <div className="flex flex-col gap-3 normal:col-span-2">
            {arrival ? (
              <Notice tone="info">
                <Localized id="gardens-arrival-posts-nothing" />
              </Notice>
            ) : (
              <GardenEntryNumbersCard numbers={preview?.numbers ?? null} empty={!filled} />
            )}
            {payment ? (
              <EntryPartsCard parts={parts} title={l10n.getString('gardens-parts-title')} />
            ) : null}
            {preview === undefined || arrival ? null : <CashNote cash={preview.cash} />}
            {warnings.length === 0 ? null : (
              <ul className="flex flex-col gap-1">
                {warnings.map((item) => (
                  <li
                    key={`${item.code}-${item.field}`}
                    role="status"
                    className="flex items-start gap-2 type-body text-warn"
                  >
                    <TriangleAlert aria-hidden="true" className="mt-1 size-icon-sm shrink-0" />
                    <span>{errorText(item)}</span>
                  </li>
                ))}
              </ul>
            )}
            {previewFailed ? (
              <ErrorState
                title={<Localized id="gardens-preview-failed" />}
                error={uiErrorOf(view.query.error)}
                onRetry={() => {
                  void view.query.refetch();
                }}
              />
            ) : null}
          </div>
        </div>
      </form>
      {flow.cashAfter === null ? null : (
        <CashConfirmDialog
          open={flow.ask}
          onOpenChange={(open) => {
            if (flow.onCashOpenChange(open)) {
              // After a "no" the cursor is on the first money field (PRD FR-6.7).
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
            // Focus goes where the screen puts it, not back to the button.
            event.preventDefault();
          }}
        />
      )}
      <SaveChangesPrompt
        open={askNew}
        onOpenChange={setAskNew}
        onSave={() => {
          void save().then((saved) => {
            if (saved) setAskNew(false);
          });
        }}
        onLeave={() => {
          setAskNew(false);
          startNew();
        }}
      />
    </div>
  );
}
