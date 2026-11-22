// A saved voucher that cannot be edited (PRD FR-15.3, FR-6.6): it was cancelled, or its day is
// closed. It shows what was stored and works nothing out again (CLAUDE.md rule 1). A cancelled one
// says so in words and in red, with the reason (NFR-USE-12). A closed day says what to do: the Owner
// reopens the day, and then the voucher can be changed. No input exists on this screen.

import { Localized } from '@fluent/react';
import { useEntryMarks } from '../../../shared/edit';
import { isoToDisplayDate } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { Card, Code, EditStampLine, Fact, Notice, Num, fieldMark } from '../../../shared/ui';
import { screenTypeOf } from '../screen-type';
import type { VoucherRow } from '../types';

export function SavedVoucherView({ row }: { row: VoucherRow }) {
  const language = useLanguage();
  const type = screenTypeOf(row.voucher_type);
  const cancelled = row.status === 'cancelled';
  // What an edit changed, marked "Before ..." on the field, and the stamp of the last edit (plan 89, PRD FR-7.71).
  const marks = useEntryMarks('voucher', row.id, row.edited !== null);
  return (
    <div className="flex flex-col gap-4">
      {cancelled ? (
        <Notice tone="error">
          <Localized id="vouchers-view-cancelled" />
          {row.cancel_reason === '' ? null : <> {row.cancel_reason}</>}
        </Notice>
      ) : (
        <Notice tone="info">
          <Localized id="vouchers-view-closed" />
        </Notice>
      )}
      {row.edited === null ? null : (
        <EditStampLine stamp={row.edited} history={{ entity: 'voucher', id: row.id }} />
      )}
      <Card compact>
        <dl className="grid grid-cols-2 gap-4 normal:grid-cols-3">
          <Fact label={<Localized id="vouchers-col-number" />}>
            {type === null ? (
              <Code>{String(row.number)}</Code>
            ) : (
              <Code>
                <Localized id={`vouchers-number-${type}`} vars={{ number: String(row.number) }} />
              </Code>
            )}
          </Fact>
          <Fact label={<Localized id="vouchers-field-date" />}>
            <Code>{isoToDisplayDate(row.entry_date)}</Code>
          </Fact>
          <Fact label={<Localized id="vouchers-col-account" />}>
            {row.account === null ? null : displayName(row.account, language)}
          </Fact>
          <Fact stat label={<Localized id="vouchers-field-amount" />}>
            <span className="flex flex-wrap items-center gap-2">
              <Num inherit paisa={row.amount_paisa} />
              {fieldMark(marks, 'amount_paisa')}
            </span>
          </Fact>
          {row.reason === '' ? null : (
            <Fact label={<Localized id="vouchers-field-reason-lent" />}>{row.reason}</Fact>
          )}
          {row.expected_goods === '' ? null : (
            <Fact label={<Localized id="vouchers-field-expected" />}>{row.expected_goods}</Fact>
          )}
          {row.narration === '' ? null : (
            <Fact label={<Localized id="vouchers-field-narration" />}>{row.narration}</Fact>
          )}
        </dl>
      </Card>
    </div>
  );
}
