// A saved voucher that cannot be edited (PRD FR-15.3, FR-6.6): it was cancelled, or its day is
// closed. It shows what was stored and works nothing out again (CLAUDE.md rule 1). A cancelled one
// says so in words and in red, with the reason (NFR-USE-12). A closed day says what to do: the Owner
// reopens the day, and then the voucher can be changed. No input exists on this screen.

import { Localized } from '@fluent/react';
import { isoToDisplayDate } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { Card, Code, Fact, Notice, Num } from '../../../shared/ui';
import { screenTypeOf } from '../screen-type';
import type { VoucherRow } from '../types';

export function SavedVoucherView({ row }: { row: VoucherRow }) {
  const language = useLanguage();
  const type = screenTypeOf(row.voucher_type);
  const cancelled = row.status === 'cancelled';
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
          {row.person === null ? null : (
            <Fact label={<Localized id="vouchers-field-person" />}>
              <span className="flex flex-wrap items-baseline gap-x-3">
                {displayName(row.person, language)}
                {row.person.phone === '' ? null : <Code>{row.person.phone}</Code>}
              </span>
            </Fact>
          )}
          <Fact label={<Localized id="vouchers-field-amount" />}>
            <Num paisa={row.amount_paisa} />
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
