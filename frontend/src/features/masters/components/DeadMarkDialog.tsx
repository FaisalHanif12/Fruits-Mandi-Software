// "ڈوبی رقم میں ڈالیں" (PRD FR-10.10, plan 40): a person who owes the shop has died or left the
// business. The box shows what he owes now (the backend's preview), asks for the date (an open
// date), the reason and a note, and moves him. Nothing posts: his account turns inactive and what
// he owes stays in the total اگراہی, shown on its own as ڈوبی رقم.
//
// - The words and the numbers are the backend's: the owed amount comes from `masters_dead_preview`,
//   and a person who cannot be moved is told why by an error code, in the app language.
// - A refusal of the save stays inside the box and keeps what was typed (PRD FR-7.6).
// - The box is opened by the app frame (app/DeadAmountHost.tsx) from the party card, the account
//   form and the rows of the dues reports. When it closes it asks every screen to read again.
// - Four states of the preview: reading, a refusal with a retry, "cannot be moved", and the form.

import { Localized, useLocalization } from '@fluent/react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect, useRef, useState } from 'react';
import type { DeadReason } from '../../../bindings/DeadReason';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { SHORTCUTS, useFormNavigation } from '../../../shared/keyboard';
import {
  Balance,
  Button,
  DateInput,
  ErrorMessage,
  ErrorNotice,
  Field,
  FormDialog,
  FreeTextInput,
  LoadingState,
  Notice,
  Num,
  ScriptText,
  Select,
  fieldError,
  showToast,
  type DeadMoveRequest,
} from '../../../shared/ui';
import { useDayStatus } from '../../../app/use-day-status';
import { deadMark, deadPreview } from '../api';

const REASONS: readonly DeadReason[] = ['died', 'left', 'other'];

interface DeadMarkDialogProps {
  request: DeadMoveRequest;
  onClose: (moved: boolean) => void;
}

export function DeadMarkDialog({ request, onClose }: DeadMarkDialogProps) {
  const language = useLanguage();
  const { l10n } = useLocalization();
  const client = useQueryClient();
  const day = useDayStatus();
  const preview = useQuery({
    queryKey: ['dead', 'preview', request.accountId, request.personKey],
    queryFn: () =>
      deadPreview({ account_id: request.accountId, pukhta_person_key: request.personKey }),
    staleTime: 0,
    gcTime: 0,
  });
  // The day typed. Null until the user types: the box then shows today, so it is visible.
  const [typedDate, setTypedDate] = useState<string | null>(null);
  const [reason, setReason] = useState<DeadReason>('died');
  const [note, setNote] = useState('');
  const today = day.data === undefined ? '' : isoToDisplayDate(day.data.today);
  const date = typedDate ?? today;
  const name = displayName(request.names, language);

  const mark = useMutation({
    mutationFn: () =>
      deadMark({
        account_id: request.accountId,
        pukhta_person_key: request.personKey,
        entry_date: date.trim(),
        reason,
        note: note.trim(),
      }),
    onSuccess: () => {
      // The tiles, the lists, the party card and the pickers all change at once.
      void client.invalidateQueries();
      showToast('success', l10n.getString('dead-move-saved', { name }));
      onClose(true);
    },
  });
  const refusal = mark.isError ? uiErrorOf(mark.error) : null;
  const data = preview.data;
  // The date box takes the focus as soon as the box has its numbers, so the user can type at once.
  const formRef = useRef<HTMLFormElement>(null);
  const ready = data !== undefined;
  useEffect(() => {
    if (!ready) return;
    const box = formRef.current?.querySelector<HTMLInputElement>('input');
    box?.focus();
    box?.select();
  }, [ready]);
  const canSave = data?.allowed === true && date.trim() !== '' && !mark.isPending;
  // Enter moves to the next box; Enter in the last box (the note) moves the person (PRD FR-7.14).
  const navigation = useFormNavigation(() => {
    if (canSave) mark.mutate();
  });

  let body;
  if (preview.isPending) {
    body = <LoadingState />;
  } else if (preview.isError) {
    body = (
      <div role="alert" className="flex flex-col gap-2 text-due">
        <Localized id="dead-move-failed">
          <p className="type-body" />
        </Localized>
        {uiErrorOf(preview.error).errors.map((item, index) => (
          <ErrorMessage key={`${item.code}-${String(index)}`} item={item}>
            <p className="type-body" />
          </ErrorMessage>
        ))}
        <div>
          <Button
            onClick={() => {
              void preview.refetch();
            }}
          >
            <Localized id="common-retry" />
          </Button>
        </div>
      </div>
    );
  } else {
    const ready = preview.data;
    body = (
      <>
        <p className="flex flex-wrap items-baseline gap-x-3 type-body text-ink">
          <Localized id="dead-move-owes">
            <span className="text-ink-muted" />
          </Localized>
          <span className="type-figure">
            <Num inherit paisa={ready.owed_paisa} />
          </span>
        </p>
        {!ready.allowed ? (
          <Notice tone="warn">
            <span className="flex flex-col gap-1">
              <Localized id="dead-move-cannot">
                <span />
              </Localized>
              {ready.reason_not_allowed === null ? null : (
                <ErrorMessage item={{ code: ready.reason_not_allowed, values: {} }}>
                  <span />
                </ErrorMessage>
              )}
            </span>
          </Notice>
        ) : null}
        {ready.linked === null ? null : (
          <Notice tone="info">
            <Localized
              id="dead-move-linked"
              vars={{
                name: displayName(ready.linked, language),
                code: ready.linked.code,
              }}
              elems={{
                name: <ScriptText />,
                num: <Num />,
                money: <Balance balance={ready.linked.balance} />,
              }}
            >
              <span />
            </Localized>
          </Notice>
        )}
        <Field
          label={<Localized id="dead-move-date" />}
          hint={
            ready.last_posting_date === null ? undefined : (
              <Localized
                id="dead-move-last-entry"
                vars={{ date: isoToDisplayDate(ready.last_posting_date) }}
                elems={{ num: <Num /> }}
              >
                <span />
              </Localized>
            )
          }
          error={fieldError(refusal, 'entry_date')}
        >
          {(control) => (
            <DateInput
              {...control}
              value={date}
              onChange={(text) => {
                setTypedDate(text);
              }}
            />
          )}
        </Field>
        <Field label={<Localized id="dead-move-reason" />} error={fieldError(refusal, 'reason')}>
          {(control) => (
            <Select
              {...control}
              value={reason}
              onChange={setReason}
              options={REASONS.map((value) => ({
                value,
                label: <Localized id={`dead-reason-${value}`} />,
              }))}
            />
          )}
        </Field>
        <Field
          label={<Localized id="dead-move-note" />}
          hint={reason === 'other' ? <Localized id="dead-move-note-hint" /> : undefined}
          error={fieldError(refusal, 'note')}
        >
          {(control) => <FreeTextInput {...control} value={note} onChange={setNote} />}
        </Field>
      </>
    );
  }

  return (
    <FormDialog
      open
      onOpenChange={(next) => {
        if (!next) onClose(false);
      }}
      title={
        <Localized id="dead-move-title" vars={{ name }} elems={{ name: <ScriptText /> }}>
          <span />
        </Localized>
      }
      description={<Localized id="dead-move-description" />}
    >
      <form
        {...navigation}
        ref={formRef}
        className="flex flex-col gap-3"
        onSubmit={(event) => {
          event.preventDefault();
        }}
      >
        {body}
        {refusal === null ? null : (
          <ErrorNotice error={refusal} fieldsShown={['entry_date', 'reason', 'note']} />
        )}
        <div className="mt-2 flex flex-wrap justify-end gap-2">
          <Button
            shortcut={SHORTCUTS.back}
            onClick={() => {
              onClose(false);
            }}
          >
            <Localized id="dead-move-back" />
          </Button>
          <Button
            variant="primary"
            busy={mark.isPending}
            disabled={!canSave}
            onClick={() => {
              mark.mutate();
            }}
          >
            <Localized id="dead-move-confirm" />
          </Button>
        </div>
      </form>
    </FormDialog>
  );
}
