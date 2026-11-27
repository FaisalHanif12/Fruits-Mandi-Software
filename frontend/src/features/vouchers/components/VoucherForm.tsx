// The fields of one voucher (PRD 15.1 to 15.4): the date, who it is with (a buyer, a seller or any
// account) with his party card, the amount with the
// buttons that fill it from the backend's figures, how the money moves, and what only some types
// have (the collector of a receipt, the optional reason of cash lent, the expected goods of an advance).
// Everything is text in the screen's state, so a language switch loses nothing (F39). The screen
// owns the keys: F10 saves, F2 starts a new voucher.
//
// Nothing is filled by guessing (CLAUDE.md rule 11): "full balance" and "oldest bill" put the
// backend's amount into the field as text, where it can be changed, and the backend reads it again
// on save.

import { Localized, useLocalization } from '@fluent/react';
import type { UiError } from '../../../bindings/UiError';
import { pickedOf, type CardChukta } from '../../../shared/accounts';
import type { EntryMarks } from '../../../shared/edit';
import { formatPaisa, useGrouping } from '../../../shared/format';
import {
  AccountPicker,
  DateInput,
  Field,
  FreeTextInput,
  LinkButton,
  MoneyInput,
  Num,
  Picker,
  SegmentedControl,
  fieldError,
  fieldMark,
  type SegmentedOption,
  FormGrid,
} from '../../../shared/ui';
import { accountPath, accountScope, helperTarget, isCashLent, pickerValueOf } from '../draft';
import { useOpenBills, useOpenPayables } from '../hooks';
import type { FromSide, VoucherDraft } from '../types';

interface VoucherFormProps {
  draft: VoucherDraft;
  onChange: (patch: Partial<VoucherDraft>) => void;
  /** What the backend refused or the preview found, with the field paths. */
  error: UiError | null;
  /** A saved voucher is being edited: its type cannot change. The reason and the sign are asked in a box on save. */
  editing: boolean;
  /** The marks of the edits this voucher went through ("Before 600", plan 44); none when new. */
  marks?: EntryMarks | undefined;
  /**
   * Which account the party card is about now. The screen draws the card itself, beside the fields (plan 89), so a
   * tall card never pushes the amount and the reason out of view; the picker only says whose card it is.
   */
  onCardAccount: (accountId: number | null, chukta: CardChukta | null | undefined) => void;
}

const FROM_OPTIONS: readonly SegmentedOption<FromSide>[] = [
  { value: 'buyer', label: <Localized id="vouchers-from-buyer" /> },
  { value: 'seller', label: <Localized id="vouchers-from-seller" /> },
];

/** The label of the account field: it says who the voucher is with, by type. */
function accountLabelId(draft: VoucherDraft, editing: boolean): string {
  if (draft.type === 'payment') return 'vouchers-field-paid-to';
  // A new advance goes to a seller or, as cash lent, to a buyer (PRD 15.4, FR-15.11); a saved one
  // that is being changed is a seller's.
  if (draft.type === 'advance') {
    return editing ? 'vouchers-field-seller' : 'vouchers-field-party';
  }
  // What is left is a receipt, from a buyer or from a seller returning advance money.
  return draft.from === 'buyer' ? 'vouchers-field-buyer' : 'vouchers-field-seller';
}

export function VoucherForm({
  draft,
  onChange,
  error,
  editing,
  marks,
  onCardAccount,
}: VoucherFormProps) {
  const { l10n } = useLocalization();
  const grouping = useGrouping();
  const target = helperTarget(draft);
  // The same reads the panels make (one request, shared by the cache): the amounts of the buttons.
  const bills = useOpenBills(
    target.kind === 'bills' && draft.type === 'receipt' ? target.accountId : null,
  );
  const payables = useOpenPayables(target.kind === 'payables' ? target.accountId : null);
  // For a seller "the whole balance" is what the shop really owes him: his payable less his open advance, never
  // below 0, as the backend works it out (PRD v1.61, plan 100 A).
  const fillFull = bills.data?.open_paisa ?? payables.data?.net_payable_paisa ?? null;
  const oldest = bills.data?.bills[0]?.open_paisa ?? null;
  const fill = (amount: NonNullable<typeof fillFull>) => {
    onChange({ amount: formatPaisa(amount, { grouping }) });
  };

  const dateField = (
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
  );
  const fromField =
    draft.type === 'receipt' ? (
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
                // The other side is another kind of account: the account and the collector of
                // this side mean nothing to it.
                onChange({
                  from,
                  account: null,
                  accountType: null,
                  collector: null,
                });
              }}
            />
          </div>
        )}
      </Field>
    ) : null;
  const accountField = (
    <div className="min-w-0">
      <Field
        label={<Localized id={accountLabelId(draft, editing)} />}
        mark={fieldMark(marks, 'account_id')}
        error={fieldError(error, accountPath(draft))}
      >
        {(props) => (
          <AccountPicker
            {...props}
            // A new key per scope: the picker starts from its own list when the side changes.
            key={[accountScope(draft, editing), draft.type].join('/')}
            scope={accountScope(draft, editing)}
            // The party card is drawn by the screen, beside the fields (plan 89).
            showCard={false}
            onCardAccount={onCardAccount}
            // A person in the dead amount can still pay: a receipt offers him, tagged "ڈوبی"
            // (PRD FR-10.10). No other entry does.
            includeDead={draft.type === 'receipt'}
            value={draft.account}
            onChange={(hit) => {
              onChange({
                account: hit === null ? null : pickerValueOf(hit),
                accountType: hit?.account_type ?? null,
                reason: '',
              });
            }}
          />
        )}
      </Field>
    </div>
  );
  const amountField = (
    <div className="flex flex-col gap-1">
      <Field
        label={<Localized id="vouchers-field-amount" />}
        mark={fieldMark(marks, 'amount_paisa')}
        error={fieldError(error, 'amount')}
      >
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
  );
  const collectorField =
    draft.type === 'receipt' ? (
      <Field
        label={<Localized id="vouchers-field-collector" />}
        mark={fieldMark(marks, 'collector_staff_id')}
        error={fieldError(error, 'collector_staff_id')}
      >
        {(props) => (
          <Picker
            {...props}
            scope="collector"
            value={draft.collector}
            onChange={(hit) => {
              onChange({
                collector: hit === null ? null : pickedOf(hit),
              });
            }}
          />
        )}
      </Field>
    ) : null;
  const reasonField = isCashLent(draft) ? (
    <Field
      label={<Localized id="vouchers-field-reason-lent" />}
      mark={fieldMark(marks, 'reason')}
      info={
        <Localized id="vouchers-reason-lent-hint">
          <p />
        </Localized>
      }
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
  ) : null;
  // The goods a seller will send: a buyer's advance is cash lent and has none (PRD 15.4).
  const expectedField =
    draft.type === 'advance' && !isCashLent(draft) ? (
      <Field
        label={<Localized id="vouchers-field-expected" />}
        mark={fieldMark(marks, 'expected_goods')}
        info={
          <Localized id="vouchers-field-expected-hint">
            <p />
          </Localized>
        }
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
    ) : null;
  const narrationField = (
    <Field
      label={<Localized id="vouchers-field-narration" />}
      mark={fieldMark(marks, 'narration')}
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
  );
  // A saved voucher that is being changed (plan 89): the amount, the thing the Owner comes to change, stands in the
  // second row beside the day and the account, so it is on screen at once; the side the money moves from (fixed for a
  // saved voucher) comes after it. The reason of the change and the sign are asked in a box when he saves (plan 92).
  if (editing) {
    return (
      <FormGrid columns={2}>
        {dateField}
        {accountField}
        {amountField}
        {fromField}
        {collectorField}
        {reasonField}
        {expectedField}
        {narrationField}
      </FormGrid>
    );
  }
  return (
    <FormGrid columns={3}>
      {dateField}
      {fromField}
      {accountField}
      {amountField}
      {collectorField}
      {reasonField}
      {expectedField}
      {narrationField}
    </FormGrid>
  );
}
