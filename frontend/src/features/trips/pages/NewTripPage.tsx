// A new trip (route /trips/new; PRD 12.10, FR-12.29, FR-7.67): the questions come in this order:
// first the shares (which partners are in it and each one's %, with the running total), then
// where each partner's money comes from (the shop's cash or his own pocket), then the details
// (the day, the market, who was sent, a note). The form starts filled from the last trip (PRD
// A56); the first trip starts from the firm's shares. The shares must total exactly 100%: the
// total is the backend's (partners_check_shares), and the save is refused otherwise. The shares
// cannot change after the trip's first entry.
//
// - Keys: F10 or Ctrl+S saves, Esc goes back to the list (asking first if there is typed work).
// - Everything typed is text in this screen's state, so a language switch loses nothing (F39).
// - Four states: loading while the defaults arrive, an error with a retry, the form, and a form
//   with no partner yet (a helpful message with a link to the partner setup).

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { useEffect, useMemo, useRef, useState } from 'react';
import { LeaveGuard } from '../../../app/LeaveGuard';
import { useDayStatus } from '../../../app/use-day-status';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { formatBps } from '../../../shared/format';
import { useFocusFirstField, useRefusalIntoView, useSingleFlight } from '../../../shared/hooks';
import { SHORTCUTS, useEscape, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  Card,
  DateInput,
  ErrorNotice,
  ErrorState,
  Field,
  FreeTextInput,
  LinkButton,
  LoadingState,
  Notice,
  PartnerSharesEditor,
  fieldError,
  showToast,
  InfoHint,
} from '../../../shared/ui';
import { SharesTotal, useSharesCheck, type SharesCheck, type TotalState } from '../../partners';
import {
  TRIP_FIELD_PATHS,
  buildTrip,
  newTripDraft,
  tripDraftChanged,
  tripShareInputs,
} from '../draft';
import { useCreateTrip, useTripDefaults } from '../hooks';
import type { TripDefaults, TripDraft } from '../types';

/** The rows of the running total, in the paths of this form (`partners[i].share`). */
function liveShareErrors(check: SharesCheck | undefined, current: boolean): UiError | null {
  if (check === undefined || !current) return null;
  const rows = check.errors.filter((item) => item.field.startsWith('shares['));
  if (rows.length === 0) return null;
  return {
    ok: false,
    errors: rows.map((item) => ({
      code: item.code,
      field: item.field.replace(/^shares/, 'partners'),
      values: item.values,
    })),
    ref: '',
  };
}

function NewTripForm({ defaults, today }: { defaults: TripDefaults; today: string }) {
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  const create = useCreateTrip();
  const flight = useSingleFlight();
  const start = useMemo(() => newTripDraft(defaults, today), [defaults, today]);
  const [draft, setDraft] = useState<TripDraft>(start);
  const [refusal, setRefusal] = useState<{ error: UiError; text: string } | null>(null);
  // The id of the trip once it is made: the leave guard is off by then, and the trip opens.
  const [createdId, setCreatedId] = useState<number | null>(null);
  const formRef = useFocusFirstField<HTMLFormElement>();
  const rootRef = useRef<HTMLDivElement>(null);

  // Only the partners' rows: typing in the market must not ask for the total again.
  const shares = useMemo(() => tripShareInputs(draft.partners), [draft.partners]);
  const { query, current } = useSharesCheck(shares);
  const check = query.data;
  const totalState: TotalState =
    check === undefined || !current ? 'checking' : check.is_100 ? 'ok' : 'bad';
  const draftText = useMemo(() => JSON.stringify(draft), [draft]);
  const shown = refusal !== null && refusal.text === draftText ? refusal.error : null;
  const error = shown ?? liveShareErrors(check, current);
  useRefusalIntoView(rootRef, shown);

  const changed = tripDraftChanged(draft, start);
  const canSave = totalState === 'ok' && draft.date.trim() !== '';

  const patchPartner = (index: number, patch: Partial<TripDraft['partners'][number]>) => {
    setDraft((previous) => ({
      ...previous,
      partners: previous.partners.map((row, place) =>
        place === index ? { ...row, ...patch } : row,
      ),
    }));
  };

  /** True when the trip was made. A refusal is shown in the form, at the fields. */
  const save = async (): Promise<boolean> => {
    if (!canSave) return false;
    return flight.run(async () => {
      try {
        const row = await create.mutateAsync(buildTrip(draft));
        setRefusal(null);
        showToast('success', l10n.getString('trips-created', { code: row.code }));
        setCreatedId(row.id);
        return true;
      } catch (failure: unknown) {
        setRefusal({ error: uiErrorOf(failure), text: draftText });
        return false;
      }
    }, false);
  };
  useEffect(() => {
    if (createdId !== null) void navigate({ to: '/trips/$id', params: { id: String(createdId) } });
  }, [createdId, navigate]);
  const saveNow = () => {
    void save();
  };
  const navigation = useFormNavigation(saveNow);
  useShortcut(SHORTCUTS.save, saveNow);
  useEscape('leave', () => {
    void navigate({ to: '/trips' });
  });

  return (
    <div ref={rootRef} className="flex max-w-page flex-col gap-4">
      <LeaveGuard dirty={changed && createdId === null} onSave={save} />
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-1">
          <Localized id="trips-new-title">
            <h1 className="sr-only normal:not-sr-only normal:type-title" />
          </Localized>
          <div className="hidden normal:block">
            <InfoHint>
              <Localized id="trips-new-help">
                <p />
              </Localized>
            </InfoHint>
          </div>
        </div>
        <Button
          variant="primary"
          shortcut={SHORTCUTS.save}
          busy={flight.busy}
          disabled={!canSave}
          onClick={saveNow}
        >
          <Localized id="trips-new-save" />
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
              ...TRIP_FIELD_PATHS.filter((path) => path !== 'partners'),
              ...draft.partners.flatMap((_, index) => [
                `partners[${String(index)}].share`,
                `partners[${String(index)}].source`,
              ]),
            ]}
          />
        )}
        <Card
          compact
          title={
            <span className="flex items-center gap-1">
              <Localized id="trips-new-shares-title" />
              {/* Where the shares came from and what they mean, behind an ⓘ (plan 38 G5). */}
              <InfoHint>
                <Localized
                  id={defaults.from_last_trip ? 'trips-new-from-last' : 'trips-new-firm-shares'}
                >
                  <p />
                </Localized>
                <Localized id="trips-new-shares-help">
                  <p />
                </Localized>
              </InfoHint>
            </span>
          }
        >
          {draft.partners.length === 0 ? (
            <Notice
              tone="warn"
              action={
                <LinkButton
                  onClick={() => {
                    void navigate({ to: '/partners/setup' });
                  }}
                >
                  <Localized id="trips-new-no-partners-link" />
                </LinkButton>
              }
            >
              <Localized id="trips-new-no-partners" />
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
            total={check === undefined || !current ? null : formatBps(check.total_bps)}
          />
          {fieldError(error, 'partners')}
        </Card>
        <Card compact title={<Localized id="trips-new-details-title" />}>
          <div className="grid grid-cols-2 gap-4">
            <Field label={<Localized id="trips-field-date" />} error={fieldError(error, 'date')}>
              {(props) => (
                <DateInput
                  {...props}
                  value={draft.date}
                  onChange={(date) => {
                    setDraft((previous) => ({ ...previous, date }));
                  }}
                />
              )}
            </Field>
            <div />
            <Field
              label={<Localized id="trips-field-market" />}
              info={
                <Localized id="trips-field-market-hint">
                  <p />
                </Localized>
              }
              error={fieldError(error, 'market')}
            >
              {(props) => (
                <FreeTextInput
                  {...props}
                  value={draft.market}
                  onChange={(market) => {
                    setDraft((previous) => ({ ...previous, market }));
                  }}
                />
              )}
            </Field>
            <Field
              label={<Localized id="trips-field-sent" />}
              error={fieldError(error, 'sent_person')}
            >
              {(props) => (
                <FreeTextInput
                  {...props}
                  value={draft.sentPerson}
                  onChange={(sentPerson) => {
                    setDraft((previous) => ({ ...previous, sentPerson }));
                  }}
                />
              )}
            </Field>
            <div className="col-span-2">
              <Field label={<Localized id="trips-field-note" />} error={fieldError(error, 'note')}>
                {(props) => (
                  <FreeTextInput
                    {...props}
                    value={draft.note}
                    onChange={(note) => {
                      setDraft((previous) => ({ ...previous, note }));
                    }}
                  />
                )}
              </Field>
            </div>
          </div>
        </Card>
      </form>
    </div>
  );
}

export function NewTripPage() {
  const day = useDayStatus();
  const defaults = useTripDefaults();
  if (day.isPending || defaults.isPending) return <LoadingState />;
  if (day.isError || defaults.isError) {
    const failure: unknown = day.isError ? day.error : defaults.error;
    return (
      <ErrorState
        title={<Localized id="trips-load-failed" />}
        error={uiErrorOf(failure)}
        onRetry={() => {
          void day.refetch();
          void defaults.refetch();
        }}
      />
    );
  }
  return <NewTripForm defaults={defaults.data} today={day.data.today} />;
}
