// A new garden (route /gardens/new; PRD 12.11, FR-12.29, FR-12.34, FR-7.67): the questions come in
// this order: first the shares (which partners are in it and each one's %, with the running
// total), then where each partner's money comes from (the shop's cash or his own pocket), then the
// details (the two names, the day, the main fruit, the rent, the end of the season, the first
// installment, a note). The form starts filled from the firm's shares or the last garden (PRD
// A56). The shares must total exactly 100%: the total is the backend's (partners_check_shares),
// and the save is refused otherwise. The shares cannot change after the garden's first entry.
//
// The form says, in words, that a part paid from a pocket posts nothing (PRD FR-12.30). The parts
// of the first installment and the cash after it come from the backend's preview, which is the
// same function as the save (Code Standards B5).
//
// - Keys: F10 or Ctrl+S saves, Esc goes back to the list (asking first if there is typed work).
// - Everything typed is text in this screen's state, so a language switch loses nothing (F39).
// - A name made only by the spelling rules is checked before the save (PRD FR-7.66).
// - Four states: loading while the defaults arrive, an error with a retry, the form, and a form
//   with no partner yet (a helpful message with a link to the partner setup).

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { useEffect, useMemo, useRef, useState } from 'react';
import { LeaveGuard } from '../../../app/LeaveGuard';
import { useDayStatus } from '../../../app/use-day-status';
import type { FieldError } from '../../../bindings/FieldError';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { formatBps } from '../../../shared/format';
import {
  useEntrySaveFlow,
  useFocusFirstField,
  useRefusalIntoView,
  useSpellingFlow,
} from '../../../shared/hooks';
import { SHORTCUTS, useEscape, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  CashConfirmDialog,
  CashNote,
  Card,
  DateInput,
  EntryPartsCard,
  ErrorNotice,
  ErrorState,
  Field,
  FreeTextInput,
  LinkButton,
  LoadingState,
  MoneyInput,
  NamePair,
  Notice,
  PartnerSharesEditor,
  Picker,
  fieldError,
  showToast,
  type PickerValue,
} from '../../../shared/ui';
import { SharesTotal, useSharesCheck, type SharesCheck, type TotalState } from '../../partners';
import { gardensCreatePreview } from '../api';
import {
  GARDEN_FIELD_PATHS,
  buildGarden,
  gardenDraftChanged,
  gardenShareInputs,
  newGardenDraft,
} from '../draft';
import { useCreateGarden, useGardenCreatePreview, useGardenDefaults } from '../hooks';
import { gardenNumberText } from '../list-columns';
import type { GardenCreated, GardenDefaults, GardenDraft, GardenInput } from '../types';

/** Codes that only say a field is empty. Held back until the user tries to save. */
const INCOMPLETE: ReadonlySet<string> = new Set(['ERR_REQUIRED', 'ERR_REASON_REQUIRED']);
/** The spelling question has its own box on save (PRD FR-7.66), so the preview never shows it. */
const SPELLING = 'ERR_SPELLING_UNCONFIRMED';

/** The rows of the running total, in the paths of this form (`partners[i].share`). */
function liveShareErrors(check: SharesCheck | undefined, current: boolean): FieldError[] {
  if (check === undefined || !current) return [];
  return check.errors
    .filter((item) => item.field.startsWith('shares['))
    .map((item) => ({
      code: item.code,
      field: item.field.replace(/^shares/, 'partners'),
      values: item.values,
    }));
}

/** The two or three sources of problems in one error, or nothing when there are none. */
function asUiError(items: readonly FieldError[]): UiError | null {
  return items.length === 0
    ? null
    : {
        ok: false,
        errors: items.map((item) => ({ code: item.code, field: item.field, values: item.values })),
        ref: '',
      };
}

/** The two names of a record a picker gave back. */
function valueOf(hit: { id: number; code: string; name_ur: string; name_en: string }): PickerValue {
  return { id: hit.id, code: hit.code, name_ur: hit.name_ur, name_en: hit.name_en };
}

function NewGardenForm({ defaults, today }: { defaults: GardenDefaults; today: string }) {
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  const create = useCreateGarden();
  const spelling = useSpellingFlow();
  const start = useMemo(() => newGardenDraft(defaults, today), [defaults, today]);
  const [draft, setDraft] = useState<GardenDraft>(start);
  const [tried, setTried] = useState(false);
  // The id of the garden once it is made: the leave guard is off by then, and the garden opens.
  const [createdId, setCreatedId] = useState<number | null>(null);
  const made = useRef(false);
  // True once the reason of the first installment was filled with the standard words.
  const [reasonFilled, setReasonFilled] = useState(false);
  const formRef = useFocusFirstField<HTMLFormElement>();
  const rootRef = useRef<HTMLDivElement>(null);

  // Only the partners' rows: typing in the name must not ask for the total again.
  const shares = useMemo(() => gardenShareInputs(draft.partners), [draft.partners]);
  const { query: sharesQuery, current: sharesCurrent } = useSharesCheck(shares);
  const check = sharesQuery.data;
  const totalState: TotalState =
    check === undefined || !sharesCurrent ? 'checking' : check.is_100 ? 'ok' : 'bad';

  // The preview is asked once a rent or a first installment is typed, so an empty form asks nothing.
  const input = useMemo(
    () =>
      draft.rent.trim() === '' && draft.paidNow.trim() === '' ? null : buildGarden(draft, false),
    [draft],
  );
  const view = useGardenCreatePreview(input);
  const preview = input === null ? undefined : view.query.data;
  const previewFailed = input !== null && view.query.isError;

  const changed = gardenDraftChanged(draft, start);
  const canSave = totalState === 'ok' && draft.date.trim() !== '';

  const draftText = useMemo(() => JSON.stringify(draft), [draft]);
  const flow = useEntrySaveFlow<GardenInput, GardenCreated | undefined>({
    draftText,
    build: (confirm) => buildGarden(draft, confirm),
    // The spelling box comes first; a "go back" there saves nothing (PRD FR-7.66).
    save: (built) =>
      spelling.guard(
        draft.names,
        (names) => {
          setDraft((previous) => ({ ...previous, names }));
        },
        (confirmedWords) => create.mutateAsync({ ...built, confirmed_rules_words: confirmedWords }),
      ),
    freshCash: (built) => gardensCreatePreview(built).then((answer) => answer.cash),
    cash: preview?.cash,
    current: view.current,
    onSaved: (saved) => {
      if (saved === undefined) return;
      made.current = true;
      showToast(
        'success',
        l10n.getString('gardens-created', {
          number: gardenNumberText(saved.row.number, (id, vars) => l10n.getString(id, vars)),
        }),
      );
      setCreatedId(saved.row.id);
    },
  });
  useEffect(() => {
    if (createdId !== null)
      void navigate({ to: '/gardens/$id', params: { id: String(createdId) } });
  }, [createdId, navigate]);

  // What the preview and the running total found, in the shape of a refused save, so the fields
  // can show it. Problems that only say "this is empty" wait until the user tries to save.
  const found = useMemo(() => {
    const fromPreview = (preview?.errors ?? []).filter(
      (item) => item.code !== SPELLING && (tried || !INCOMPLETE.has(item.code)),
    );
    const live = liveShareErrors(check, sharesCurrent).filter(
      (row) => !fromPreview.some((item) => item.field === row.field && item.code === row.code),
    );
    return asUiError([...fromPreview, ...live]);
  }, [preview?.errors, tried, check, sharesCurrent]);
  const error = flow.refusal ?? found;
  useRefusalIntoView(rootRef, flow.refusal);

  const patchPartner = (index: number, patch: Partial<GardenDraft['partners'][number]>) => {
    setDraft((previous) => ({
      ...previous,
      partners: previous.partners.map((row, place) =>
        place === index ? { ...row, ...patch } : row,
      ),
    }));
  };
  const change = (patch: Partial<GardenDraft>) => {
    setDraft((previous) => ({ ...previous, ...patch }));
    setTried(false);
  };

  /** True when the garden was made. A refusal is shown in the form, at the fields. */
  const save = async (): Promise<boolean> => {
    if (!canSave) return false;
    setTried(true);
    made.current = false;
    await flow.save();
    return made.current;
  };
  const saveNow = () => {
    void save();
  };
  const navigation = useFormNavigation(saveNow);
  useShortcut(SHORTCUTS.save, saveNow);
  useEscape('leave', () => {
    void navigate({ to: '/gardens' });
  });

  return (
    <div ref={rootRef} className="flex max-w-page flex-col gap-4">
      <LeaveGuard dirty={changed && createdId === null} onSave={save} />
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex flex-col gap-1">
          <Localized id="gardens-new-title">
            <h1 className="sr-only normal:not-sr-only normal:type-title" />
          </Localized>
          <Localized id="gardens-new-help">
            <p className="hidden type-body text-ink-muted normal:block" />
          </Localized>
        </div>
        <Button
          variant="primary"
          shortcut={SHORTCUTS.save}
          busy={flow.busy}
          disabled={!canSave}
          onClick={saveNow}
        >
          <Localized id="gardens-new-save" />
        </Button>
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
          <ErrorNotice
            error={error}
            fieldsShown={[
              ...GARDEN_FIELD_PATHS.filter((path) => path !== 'partners'),
              ...draft.partners.flatMap((_, index) => [
                `partners[${String(index)}].share`,
                `partners[${String(index)}].source`,
              ]),
            ]}
          />
        )}
        <Card compact title={<Localized id="gardens-new-shares-title" />}>
          <Localized id="gardens-new-from-defaults">
            <p className="type-body text-ink-muted" />
          </Localized>
          <Localized id="gardens-new-shares-help">
            <p className="type-body text-ink-muted" />
          </Localized>
          {draft.partners.length === 0 ? (
            <Notice
              tone="warn"
              action={
                <LinkButton
                  onClick={() => {
                    void navigate({ to: '/partners/setup' });
                  }}
                >
                  <Localized id="gardens-new-no-partners-link" />
                </LinkButton>
              }
            >
              <Localized id="gardens-new-no-partners" />
            </Notice>
          ) : (
            <PartnerSharesEditor
              rows={draft.partners}
              error={error}
              onShareChange={(index, share) => {
                patchPartner(index, { share });
              }}
              onSourceChange={(index, source) => {
                patchPartner(index, { source });
              }}
            />
          )}
          <SharesTotal
            state={totalState}
            total={check === undefined || !sharesCurrent ? null : formatBps(check.total_bps)}
          />
          {fieldError(error, 'partners')}
          <Localized id="gardens-new-sources-help">
            <p className="type-body text-ink-muted" />
          </Localized>
        </Card>
        <Card compact title={<Localized id="gardens-new-details-title" />}>
          <div className="grid grid-cols-2 gap-4">
            <div className="col-span-2">
              <NamePair
                value={draft.names}
                onChange={(names) => {
                  change({ names });
                }}
                urduLabel={<Localized id="gardens-field-name-ur" />}
                englishLabel={<Localized id="gardens-field-name-en" />}
                urduError={fieldError(error, 'name_ur')}
                englishError={fieldError(error, 'name_en')}
              />
            </div>
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
            <Field
              label={<Localized id="glossary-qism-maal" />}
              hint={<Localized id="gardens-field-item-hint" />}
              error={fieldError(error, 'item_id')}
            >
              {(props) => (
                <Picker
                  {...props}
                  scope="item"
                  value={draft.item}
                  onChange={(hit) => {
                    change({ item: hit === null ? null : valueOf(hit) });
                  }}
                />
              )}
            </Field>
            <Field
              label={<Localized id="gardens-field-rent" />}
              hint={<Localized id="gardens-field-rent-hint" />}
              error={fieldError(error, 'rent')}
            >
              {(props) => (
                <MoneyInput
                  {...props}
                  value={draft.rent}
                  onChange={(rent) => {
                    change({ rent });
                  }}
                />
              )}
            </Field>
            <Field
              label={<Localized id="gardens-field-season-end" />}
              error={fieldError(error, 'season_end')}
            >
              {(props) => (
                <DateInput
                  {...props}
                  value={draft.seasonEnd}
                  onChange={(seasonEnd) => {
                    change({ seasonEnd });
                  }}
                />
              )}
            </Field>
            <div data-paid-now="" className="col-span-2">
              <Field
                label={<Localized id="gardens-field-paid-now" />}
                hint={<Localized id="gardens-field-paid-now-hint" />}
                error={fieldError(error, 'paid_now')}
              >
                {(props) => (
                  <MoneyInput
                    {...props}
                    value={draft.paidNow}
                    onChange={(paidNow) => {
                      // The reason is filled with the standard words once, like a typed value, and
                      // stays visible and changeable (Code Standards F43, NFR-USE-27).
                      const fillReason =
                        paidNow.trim() !== '' && !reasonFilled && draft.paidNowReason === '';
                      if (fillReason) setReasonFilled(true);
                      change({
                        paidNow,
                        ...(fillReason
                          ? { paidNowReason: l10n.getString('gardens-first-installment-reason') }
                          : {}),
                      });
                    }}
                  />
                )}
              </Field>
            </div>
            {draft.paidNow.trim() === '' ? null : (
              <div className="col-span-2">
                <Field
                  label={<Localized id="gardens-field-paid-now-reason" />}
                  hint={<Localized id="gardens-field-paid-now-reason-hint" />}
                  error={fieldError(error, 'paid_now_reason')}
                >
                  {(props) => (
                    <FreeTextInput
                      {...props}
                      value={draft.paidNowReason}
                      onChange={(paidNowReason) => {
                        change({ paidNowReason });
                      }}
                    />
                  )}
                </Field>
              </div>
            )}
            <div className="col-span-2">
              <Field
                label={<Localized id="gardens-field-note" />}
                error={fieldError(error, 'note')}
              >
                {(props) => (
                  <FreeTextInput
                    {...props}
                    value={draft.note}
                    onChange={(note) => {
                      change({ note });
                    }}
                  />
                )}
              </Field>
            </div>
          </div>
        </Card>
        {draft.paidNow.trim() === '' ? null : (
          <>
            <EntryPartsCard
              parts={preview?.first_installment ?? null}
              title={l10n.getString('gardens-first-parts-title')}
            />
            {preview === undefined ? null : <CashNote cash={preview.cash} />}
          </>
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
      </form>
      {flow.cashAfter === null ? null : (
        <CashConfirmDialog
          open={flow.ask}
          onOpenChange={(open) => {
            if (flow.onCashOpenChange(open)) {
              // After a "no" the cursor is on the money field of the first installment (FR-6.7).
              window.requestAnimationFrame(() => {
                formRef.current?.querySelector<HTMLInputElement>('[data-paid-now] input')?.focus();
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
      {spelling.dialog}
    </div>
  );
}

export function NewGardenPage() {
  const day = useDayStatus();
  const defaults = useGardenDefaults();
  if (day.isPending || defaults.isPending) return <LoadingState />;
  if (day.isError || defaults.isError) {
    const failure: unknown = day.isError ? day.error : defaults.error;
    return (
      <ErrorState
        title={<Localized id="gardens-load-failed" />}
        error={uiErrorOf(failure)}
        onRetry={() => {
          void day.refetch();
          void defaults.refetch();
        }}
      />
    );
  }
  return <NewGardenForm defaults={defaults.data} today={day.data.today} />;
}
