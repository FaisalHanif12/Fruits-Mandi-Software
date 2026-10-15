// The partner setup: the partners with a share box each, the day the shares start, the running
// total, and Save (PRD 17.1, FR-17.1, FR-17.2). One form for two homes: the setup page and the
// first-run wizard (NFR-INS-2). The host gives it the words of its Save button and what comes
// after a save; the form is the same.
//
// - The shares are text as typed. The running total and every row's problem come from the backend
//   (partners_check_shares), and Save stays off until the total is exactly 100 (the backend checks
//   again on save). Nothing is added in the screen (CLAUDE.md rule 1).
// - What is typed lives in a store above the screen (setup-store), so a language switch, a dialog
//   or a wizard step loses nothing (Code Standards F39).
// - Keys: arrows move between the share boxes, Enter on the last field and F10 save, F2 adds a
//   partner, Esc closes a dialog.
// - Four states: loading, error with a retry, empty with a button, and the list.

import { Localized, useLocalization } from '@fluent/react';
import { Users } from 'lucide-react';
import { useMemo, useState, type ReactNode, type RefObject } from 'react';
import { LeaveGuard } from '../../../app/LeaveGuard';
import { useDayStatus } from '../../../app/use-day-status';
import { uiErrorOf } from '../../../shared/api';
import type { UiError } from '../../../bindings/UiError';
import { formatBps, isoToDisplayDate } from '../../../shared/format';
import { useFocusFirstField, useSingleFlight } from '../../../shared/hooks';
import { SHORTCUTS, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  Card,
  DateInput,
  EmptyState,
  ErrorNotice,
  ErrorState,
  Field,
  LoadingState,
  Num,
  fieldError,
  showToast,
} from '../../../shared/ui';
import { usePartnerList, useSetShares, useSharesCheck } from '../hooks';
import { activePartners, hasUnsavedShares, shareInputs, shareText } from '../setup-logic';
import { usePartnerSetupStore } from '../setup-store';
import type { PartnerRow, SharesCheck } from '../types';
import { AddPartnerDialog } from './AddPartnerDialog';
import { EditPartnerDialog } from './EditPartnerDialog';
import { ShareRow } from './ShareRow';
import { SharesTotal, type TotalState } from './SharesTotal';

const NO_PARTNERS: readonly PartnerRow[] = [];
const SHARE_FIELD = /^shares\[(\d+)]\.share$/;

interface PartnerSetupFormProps {
  /** The Fluent message of the Save button. */
  labelId: string;
  /** Runs after the shares were saved. */
  onSaved: () => void;
  /** Buttons at the start of the action row, for example the wizard's "do this later". */
  actionsStart?: ReactNode;
  /** Ask "Save changes?" before the user leaves with unsaved shares (the page, not the wizard). */
  leaveGuard?: boolean;
}

/** The problems of the live check that belong to a share box the user has typed in. */
function liveRowErrors(check: SharesCheck | undefined, typed: readonly string[]): UiError | null {
  if (check === undefined) return null;
  const errors = check.errors.filter((item) => {
    const index = SHARE_FIELD.exec(item.field)?.[1];
    return index !== undefined && (typed[Number(index)] ?? '').trim() !== '';
  });
  return errors.length === 0 ? null : { ok: false, errors, ref: '' };
}

export function PartnerSetupForm({
  labelId,
  onSaved,
  actionsStart,
  leaveGuard = false,
}: PartnerSetupFormProps) {
  const { l10n } = useLocalization();
  const list = usePartnerList();
  const day = useDayStatus();
  const edits = usePartnerSetupStore((state) => state.shareEdits);
  const fromEdit = usePartnerSetupStore((state) => state.fromEdit);
  const setShare = usePartnerSetupStore((state) => state.setShare);
  const setFrom = usePartnerSetupStore((state) => state.setFrom);
  const clear = usePartnerSetupStore((state) => state.clear);
  const [adding, setAdding] = useState(false);
  const [editing, setEditing] = useState<number | null>(null);

  const partners = list.data ?? NO_PARTNERS;
  const active = useMemo(() => activePartners(partners), [partners]);
  const inactive = useMemo(() => partners.filter((partner) => !partner.is_active), [partners]);
  const shares = useMemo(() => shareInputs(partners, edits), [partners, edits]);
  const { query: checked, current } = useSharesCheck(shares);
  const setShares = useSetShares();
  const container = useFocusFirstField<HTMLDivElement>(list.isSuccess);
  const dialogOpen = adding || editing !== null;
  // After a partner is added, focus goes to the first share box, to type the shares next.
  const firstShareBox = useMemo<RefObject<HTMLElement | null>>(
    () => ({
      get current() {
        return container.current?.querySelector<HTMLElement>('input') ?? null;
      },
    }),
    [container],
  );

  const today = day.data === undefined ? '' : isoToDisplayDate(day.data.today);
  const from = fromEdit ?? today;
  const saveError = setShares.error === null ? null : uiErrorOf(setShares.error);
  const typed = shares.map((row) => row.share);
  // Only an answer for the shares on screen: a late answer for an earlier text is not shown.
  const live = current ? liveRowErrors(checked.data, typed) : null;
  const rowPaths = shares.map((_, index) => `shares[${String(index)}].share`);

  const check = checked.data;
  const totalState: TotalState =
    check === undefined || !current ? 'checking' : check.is_100 ? 'ok' : 'bad';
  const canSave = totalState === 'ok' && from.trim() !== '';

  const flight = useSingleFlight();
  // True when the shares were saved. A refusal is shown in the form, at the fields.
  const save = async (): Promise<boolean> => {
    try {
      await setShares.mutateAsync({ effective_from: from, shares });
      showToast(
        'success',
        <Localized id="partners-saved" vars={{ date: from }} elems={{ num: <Num /> }}>
          <span />
        </Localized>,
      );
      clear();
      onSaved();
      return true;
    } catch {
      return false;
    }
  };
  const saveNow = () => {
    if (!canSave) return;
    void flight.run(save, false);
  };
  const dirty = hasUnsavedShares(partners, edits, fromEdit);
  const navigation = useFormNavigation(saveNow);
  useShortcut(SHORTCUTS.save, saveNow, { enabled: !dialogOpen });
  useShortcut(
    SHORTCUTS.newEntry,
    () => {
      setAdding(true);
    },
    { enabled: !dialogOpen },
  );

  const addButton = (primary: boolean) => (
    <Button
      variant={primary ? 'primary' : 'secondary'}
      shortcut={SHORTCUTS.newEntry}
      onClick={() => {
        setAdding(true);
      }}
    >
      <Localized id="partners-add" />
    </Button>
  );

  let body: ReactNode;
  if (list.isPending || day.isPending) {
    body = <LoadingState />;
  } else if (list.isError || day.isError) {
    const failure = list.isError ? list.error : day.error;
    body = (
      <ErrorState
        title={<Localized id="partners-setup-failed" />}
        error={uiErrorOf(failure)}
        onRetry={() => {
          void list.refetch();
          void day.refetch();
        }}
      />
    );
  } else if (active.length === 0) {
    body = (
      <EmptyState
        icon={Users}
        title={<Localized id="partners-setup-empty" />}
        help={<Localized id="partners-setup-empty-help" />}
        action={addButton(true)}
      />
    );
  } else {
    body = (
      <div ref={container} {...navigation} className="flex flex-col gap-4">
        {saveError === null ? null : (
          <ErrorNotice error={saveError} fieldsShown={[...rowPaths, 'effective_from']} />
        )}
        <Card>
          <div className="flex flex-wrap items-center justify-between gap-3">
            <Localized id="partners-list-label">
              <h2 className="type-section" />
            </Localized>
            {addButton(false)}
          </div>
          <div>
            <div className="grid grid-cols-3 gap-4 pb-2 text-ink-muted">
              <Localized id="partners-column-partner">
                <span className="type-body col-span-2" />
              </Localized>
              <Localized id="partners-column-share">
                <span className="type-body" />
              </Localized>
            </div>
            <ul aria-label={l10n.getString('partners-list-label')}>
              {active.map((partner, index) => {
                const path = rowPaths[index] ?? '';
                return (
                  <ShareRow
                    key={partner.id}
                    partner={partner}
                    share={shareText(partner, partners, edits)}
                    onShareChange={(text) => {
                      setShare(partner.id, text);
                    }}
                    error={fieldError(saveError, path) ?? fieldError(live, path)}
                    onDetails={() => {
                      setEditing(partner.id);
                    }}
                  />
                );
              })}
            </ul>
          </div>
          <div className="flex flex-wrap items-start justify-between gap-4">
            <Localized id="partners-blank-hint">
              <p className="type-body min-w-0 flex-1 text-ink-muted" />
            </Localized>
            <div className="w-full max-w-field">
              <SharesTotal
                state={totalState}
                total={check === undefined ? null : formatBps(check.total_bps)}
              />
            </div>
          </div>
        </Card>
        <Card>
          <div className="max-w-field">
            <Field
              label={<Localized id="partners-field-from" />}
              hint={<Localized id="partners-field-from-hint" />}
              error={fieldError(saveError, 'effective_from')}
            >
              {(control) => <DateInput {...control} value={from} onChange={setFrom} />}
            </Field>
          </div>
        </Card>
        {inactive.length === 0 ? null : (
          <Card title={<Localized id="partners-inactive-title" />}>
            <ul>
              {inactive.map((partner) => (
                <ShareRow
                  key={partner.id}
                  partner={partner}
                  share={null}
                  onShareChange={() => undefined}
                  onDetails={() => {
                    setEditing(partner.id);
                  }}
                />
              ))}
            </ul>
          </Card>
        )}
      </div>
    );
  }

  const showSave = list.isSuccess && active.length > 0;
  return (
    <div className="flex flex-col gap-4">
      {leaveGuard ? (
        <LeaveGuard
          dirty={dirty}
          onSave={() => (canSave ? flight.run(save, false) : Promise.resolve(false))}
        />
      ) : null}
      {body}
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-wrap gap-2">{actionsStart}</div>
        {showSave ? (
          <Button
            variant="primary"
            shortcut={SHORTCUTS.save}
            disabled={!canSave}
            busy={flight.busy || setShares.isPending}
            onClick={saveNow}
          >
            <Localized id={labelId} />
          </Button>
        ) : null}
      </div>
      <AddPartnerDialog open={adding} onOpenChange={setAdding} returnFocusTo={firstShareBox} />
      <EditPartnerDialog
        partnerId={editing}
        onClose={() => {
          setEditing(null);
        }}
      />
    </div>
  );
}
