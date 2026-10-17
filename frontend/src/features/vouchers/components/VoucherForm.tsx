// The fields of one voucher (PRD 15.1 to 15.4): the date, who it is with (a buyer, a seller or any
// account) with his party card, the one-time person for a پختہ کھاتہ account, the amount with the
// buttons that fill it from the backend's figures, how the money moves, and what only some types
// have (the collector of a receipt, the reason of cash lent, the expected goods of an advance).
// Everything is text in the screen's state, so a language switch loses nothing (F39). The screen
// owns the keys: F10 saves, F2 starts a new voucher.
//
// Nothing is filled by guessing (CLAUDE.md rule 11): "full balance" and "oldest bill" put the
// backend's amount into the field as text, where it can be changed, and the backend reads it again
// on save.

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import type { PukhtaPerson } from '../../../bindings/PukhtaPerson';
import type { UiError } from '../../../bindings/UiError';
import { formatPaisa, useGrouping } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import {
  AccountPicker,
  Button,
  Code,
  DateInput,
  Field,
  FreeTextInput,
  LinkButton,
  MoneyInput,
  Num,
  Picker,
  SegmentedControl,
  fieldError,
  type SegmentedOption,
} from '../../../shared/ui';
import { MakeRegularDialog, PukhtaPersonPicker } from '../../pukhta_khata';
import type { MadeRegular } from '../../../bindings/MadeRegular';
import {
  accountPath,
  accountScope,
  helperTarget,
  isCashLent,
  needsPerson,
  personSide,
  pickerValueOf,
} from '../draft';
import { useMakeRegular, useOpenBills, useOpenPayables } from '../hooks';
import type { FromSide, VoucherDraft } from '../types';
import { PaymentModeFields } from './PaymentModeFields';

interface VoucherFormProps {
  draft: VoucherDraft;
  onChange: (patch: Partial<VoucherDraft>) => void;
  /** What the backend refused or the preview found, with the field paths. */
  error: UiError | null;
  /** A saved voucher is being edited: its type cannot change, and a reason for the change is asked. */
  editing: boolean;
  editReason: string;
  onEditReason: (reason: string) => void;
  /** The one-time person got a regular account: the account comes back into the form. */
  onMadeRegular: (made: MadeRegular) => void;
}

const FROM_OPTIONS: readonly SegmentedOption<FromSide>[] = [
  { value: 'buyer', label: <Localized id="vouchers-from-buyer" /> },
  { value: 'seller', label: <Localized id="vouchers-from-seller" /> },
];

/** The label of the account field: it says who the voucher is with, by type. */
function accountLabelId(draft: VoucherDraft): string {
  if (draft.type === 'payment') return 'vouchers-field-paid-to';
  return draft.type === 'receipt' && draft.from === 'buyer'
    ? 'vouchers-field-buyer'
    : 'vouchers-field-seller';
}

export function VoucherForm({
  draft,
  onChange,
  error,
  editing,
  editReason,
  onEditReason,
  onMadeRegular,
}: VoucherFormProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const grouping = useGrouping();
  const [regularOpen, setRegularOpen] = useState(false);
  const target = helperTarget(draft);
  // The same reads the panels make (one request, shared by the cache): the amounts of the buttons.
  const makeRegular = useMakeRegular();
  const bills = useOpenBills(
    target.kind === 'bills' && draft.type === 'receipt' ? target.accountId : null,
    target.kind === 'bills' ? target.personKey : null,
  );
  const payables = useOpenPayables(
    target.kind === 'payables' ? target.accountId : null,
    target.kind === 'payables' ? target.personKey : null,
  );
  const fillFull = bills.data?.open_paisa ?? payables.data?.open_paisa ?? null;
  const oldest = bills.data?.bills[0]?.open_paisa ?? null;
  const fill = (amount: NonNullable<typeof fillFull>) => {
    onChange({ amount: formatPaisa(amount, { grouping }) });
  };

  const personPicked = (person: PukhtaPerson) => {
    onChange({
      person: {
        name_ur: person.name_ur,
        name_en: person.name_en,
        phone: person.phone,
        person_key: person.person_key,
      },
      personText: displayName(person, language),
    });
  };

  return (
    <div className="grid grid-cols-2 gap-4">
      <Field label={<Localized id="vouchers-field-date" />} error={fieldError(error, 'date')}>
        {(props) => (
          <DateInput
            {...props}
            value={draft.date}
            onChange={(date) => {
              onChange({ date });
            }}
          />
        )}
      </Field>
      {draft.type === 'receipt' ? (
        <Field label={<Localized id="vouchers-field-from" />}>
          {() => (
            <div>
              <SegmentedControl
                options={FROM_OPTIONS}
                value={draft.from}
                groupLabel={l10n.getString('vouchers-field-from')}
                disabled={editing}
                onChange={(from) => {
                  if (from === draft.from) return;
                  // The other side is another kind of account: the account, the person and the
                  // reason of this side mean nothing to it.
                  onChange({
                    from,
                    account: null,
                    accountType: null,
                    person: null,
                    personText: '',
                    collector: null,
                  });
                }}
              />
            </div>
          )}
        </Field>
      ) : null}
      <div className="col-span-2">
        <Field
          label={<Localized id={accountLabelId(draft)} />}
          error={fieldError(error, accountPath(draft))}
        >
          {(props) => (
            <AccountPicker
              {...props}
              // A new key per scope: the picker starts from its own list when the side changes.
              key={accountScope(draft)}
              scope={accountScope(draft)}
              value={draft.account}
              onChange={(hit) => {
                onChange({
                  account: hit === null ? null : pickerValueOf(hit),
                  accountType: hit?.account_type ?? null,
                  person: null,
                  personText: '',
                  reason: '',
                });
              }}
            />
          )}
        </Field>
      </div>
      {needsPerson(draft) ? (
        <div className="flex flex-col gap-2 col-span-2">
          <Field
            label={<Localized id="vouchers-field-person" />}
            hint={<Localized id="vouchers-person-hint" />}
            error={fieldError(error, 'person_key')}
          >
            {(props) => (
              <PukhtaPersonPicker
                {...props}
                side={personSide(draft)}
                text={draft.personText}
                onTextChange={(personText) => {
                  onChange({ personText, person: null });
                }}
                onPick={personPicked}
              />
            )}
          </Field>
          {draft.person === null ? null : (
            <div className="flex flex-wrap items-center justify-between gap-3 rounded-2 border border-border bg-surface-sunken p-3">
              <span className="flex flex-wrap items-baseline gap-x-3">
                <span className="type-field">{displayName(draft.person, language)}</span>
                {draft.person.phone === '' ? null : <Code>{draft.person.phone}</Code>}
              </span>
              <Button
                onClick={() => {
                  setRegularOpen(true);
                }}
              >
                <Localized id="vouchers-make-regular" />
              </Button>
              <MakeRegularDialog
                open={regularOpen}
                onOpenChange={setRegularOpen}
                person={draft.person}
                side={personSide(draft)}
                onMake={makeRegular.mutateAsync}
                onDone={onMadeRegular}
              />
            </div>
          )}
        </div>
      ) : null}
      <div className="flex flex-col gap-1">
        <Field label={<Localized id="vouchers-field-amount" />} error={fieldError(error, 'amount')}>
          {(props) => (
            <MoneyInput
              {...props}
              value={draft.amount}
              onChange={(amount) => {
                onChange({ amount });
              }}
            />
          )}
        </Field>
        {fillFull === null ? null : (
          <span className="flex flex-wrap items-center gap-x-4">
            <LinkButton
              onClick={() => {
                fill(fillFull);
              }}
            >
              <Localized id="vouchers-fill-full" />
              &nbsp;
              <Num paisa={fillFull} />
            </LinkButton>
            {oldest === null ? null : (
              <LinkButton
                onClick={() => {
                  fill(oldest);
                }}
              >
                <Localized id="vouchers-fill-oldest" />
                &nbsp;
                <Num paisa={oldest} />
              </LinkButton>
            )}
          </span>
        )}
      </div>
      <PaymentModeFields value={draft} error={error} onChange={onChange} />
      {draft.type === 'receipt' ? (
        <Field
          label={<Localized id="vouchers-field-collector" />}
          error={fieldError(error, 'collector_staff_id')}
        >
          {(props) => (
            <Picker
              {...props}
              scope="staff"
              value={draft.collector}
              onChange={(hit) => {
                onChange({
                  collector:
                    hit === null
                      ? null
                      : { id: hit.id, code: hit.code, name_ur: hit.name_ur, name_en: hit.name_en },
                });
              }}
            />
          )}
        </Field>
      ) : null}
      {isCashLent(draft) ? (
        <Field
          label={<Localized id="vouchers-field-reason-lent" />}
          hint={<Localized id="vouchers-reason-lent-hint" />}
          error={fieldError(error, 'reason')}
        >
          {(props) => (
            <FreeTextInput
              {...props}
              value={draft.reason}
              onChange={(reason) => {
                onChange({ reason });
              }}
            />
          )}
        </Field>
      ) : null}
      {draft.type === 'advance' ? (
        <Field
          label={<Localized id="vouchers-field-expected" />}
          hint={<Localized id="vouchers-field-expected-hint" />}
          error={fieldError(error, 'expected_goods')}
        >
          {(props) => (
            <FreeTextInput
              {...props}
              value={draft.expected_goods}
              onChange={(expected_goods) => {
                onChange({ expected_goods });
              }}
            />
          )}
        </Field>
      ) : null}
      <Field
        label={<Localized id="vouchers-field-narration" />}
        error={fieldError(error, 'narration')}
      >
        {(props) => (
          <FreeTextInput
            {...props}
            value={draft.narration}
            onChange={(narration) => {
              onChange({ narration });
            }}
          />
        )}
      </Field>
      {editing ? (
        <div className="col-span-2">
          <Field
            label={<Localized id="vouchers-field-edit-reason" />}
            hint={<Localized id="vouchers-edit-reason-hint" />}
            error={fieldError(error, 'edit_reason')}
          >
            {(props) => <FreeTextInput {...props} value={editReason} onChange={onEditReason} />}
          </Field>
        </div>
      ) : null}
    </div>
  );
}
