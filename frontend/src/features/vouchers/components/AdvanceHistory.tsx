// A seller's advances under his party card (PRD FR-7.60): how much is open and the date and
// amount of the last one, then each open advance with how much of it is adjusted and how much is
// still open. An advance is a پیشگی voucher or an opening debit marked as an advance. Everything is
// the backend's; nothing is added up here. Four states: loading, error with a retry, nothing yet,
// and the list.

import { Localized } from '@fluent/react';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate } from '../../../shared/format';
import { Code, ErrorState, LoadingState, Num } from '../../../shared/ui';
import { useAdvances } from '../hooks';

interface AdvanceHistoryProps {
  sellerId: number;
}

export function AdvanceHistory({ sellerId }: AdvanceHistoryProps) {
  const advances = useAdvances(sellerId);
  if (advances.isPending) return <LoadingState />;
  if (advances.isError) {
    return (
      <ErrorState
        title={<Localized id="vouchers-advances-failed" />}
        error={uiErrorOf(advances.error)}
        onRetry={() => {
          void advances.refetch();
        }}
      />
    );
  }
  const { open, open_paisa, last } = advances.data;
  return (
    <section className="flex flex-col gap-3 rounded-3 border border-border bg-surface p-4">
      <Localized id="vouchers-advances-title">
        <h3 className="type-section" />
      </Localized>
      {open.length === 0 && last === null ? (
        <Localized id="vouchers-advances-none">
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : (
        <>
          <p className="flex flex-wrap items-baseline gap-x-6 gap-y-1 type-body">
            <span className="flex items-baseline gap-2">
              <Localized id="vouchers-advances-open" />
              <Num paisa={open_paisa} />
            </span>
            {last === null ? null : (
              <span className="flex items-baseline gap-2">
                <Localized id="vouchers-advances-last" />
                <Num>{isoToDisplayDate(last.entry_date)}</Num>
                <Num paisa={last.amount_paisa} />
              </span>
            )}
          </p>
          {open.length === 0 ? null : (
            <table className="w-full border-collapse type-body">
              <thead>
                <tr className="text-ink-muted">
                  <th scope="col" className="pb-2 pe-3 text-start font-normal">
                    <Localized id="vouchers-col-date" />
                  </th>
                  <th scope="col" className="pb-2 pe-3 text-start font-normal">
                    <Localized id="vouchers-col-number" />
                  </th>
                  <th scope="col" className="pb-2 pe-3 text-end font-normal">
                    <Localized id="vouchers-col-amount" />
                  </th>
                  <th scope="col" className="pb-2 pe-3 text-end font-normal">
                    <Localized id="vouchers-col-adjusted" />
                  </th>
                  <th scope="col" className="pb-2 text-end font-normal">
                    <Localized id="vouchers-col-open" />
                  </th>
                </tr>
              </thead>
              <tbody>
                {open.map((item) => (
                  <tr
                    key={`${item.source}-${String(item.voucher_number)}-${item.entry_date}`}
                    className="border-t border-border"
                  >
                    <td className="py-2 pe-3">
                      <Num>{isoToDisplayDate(item.entry_date)}</Num>
                    </td>
                    <td className="py-2 pe-3">
                      {item.source === 'opening' || item.voucher_number === null ? (
                        <Localized id="vouchers-source-opening" />
                      ) : (
                        <Code>
                          <Localized
                            id="vouchers-number-advance"
                            vars={{ number: String(item.voucher_number) }}
                          />
                        </Code>
                      )}
                    </td>
                    <td className="py-2 pe-3 text-end">
                      <Num paisa={item.given_paisa} />
                    </td>
                    <td className="py-2 pe-3 text-end">
                      <Num paisa={item.adjusted_paisa} />
                    </td>
                    <td className="py-2 text-end">
                      <Num paisa={item.open_paisa} />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </>
      )}
    </section>
  );
}
