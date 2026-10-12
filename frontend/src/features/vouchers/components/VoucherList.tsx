// The running list of vouchers waiting to be saved together (PRD FR-15.1): the seller, the amount,
// the balance after, and the state of each (waiting, saved with its number, or refused with what
// to do). The total is the backend's. Each voucher is its own posting event, so a refused one
// stays here with its message while the others are saved. Four states of the whole screen live in
// the page; this list shows nothing when it is empty.

import { Localized } from '@fluent/react';
import { Trash2 } from 'lucide-react';
import { ErrorMessage } from '../../../shared/ui/ErrorMessage';
import { displayName, useLanguage } from '../../../shared/i18n';
import { Balance, Button, Card, Code, Num } from '../../../shared/ui';
import type { Paisa } from '../../../shared/format';
import type { ListedVoucher } from '../draft';
import type { AdvancePreviewRow } from '../types';

interface VoucherListProps {
  vouchers: readonly ListedVoucher[];
  /** The backend's view of each voucher by its id, for the amount and the balance after. */
  previews: ReadonlyMap<string, AdvancePreviewRow>;
  /** The total of the list as the backend worked it out. Null while there is none. */
  total: Paisa | null;
  onRemove: (id: string) => void;
  disabled: boolean;
}

export function VoucherList({ vouchers, previews, total, onRemove, disabled }: VoucherListProps) {
  const language = useLanguage();
  if (vouchers.length === 0) return null;
  return (
    <Card title={<Localized id="vouchers-list-title" />}>
      <table className="w-full border-collapse type-body">
        <thead>
          <tr className="text-ink-muted">
            <th scope="col" className="pb-2 pe-3 text-start font-normal">
              <Localized id="vouchers-col-seller" />
            </th>
            <th scope="col" className="pb-2 pe-3 text-end font-normal">
              <Localized id="vouchers-col-amount" />
            </th>
            <th scope="col" className="pb-2 pe-3 text-end font-normal">
              <Localized id="vouchers-col-new-balance" />
            </th>
            <th scope="col" className="pb-2 pe-3 text-start font-normal">
              <Localized id="vouchers-col-state" />
            </th>
            <th scope="col" className="pb-2 font-normal">
              <span className="sr-only">
                <Localized id="vouchers-remove" />
              </span>
            </th>
          </tr>
        </thead>
        <tbody>
          {vouchers.map((voucher) => {
            const preview = previews.get(voucher.id);
            const { status } = voucher;
            return (
              <tr key={voucher.id} className="border-t border-border align-top">
                <td className="py-2 pe-3">
                  {voucher.draft.seller === null ? null : (
                    <span className="flex flex-wrap items-baseline gap-x-2">
                      <Code>{voucher.draft.seller.code}</Code>
                      <span>{displayName(voucher.draft.seller, language)}</span>
                    </span>
                  )}
                </td>
                <td className="py-2 pe-3 text-end">
                  {preview === undefined ? null : <Num paisa={preview.amount_paisa} />}
                </td>
                <td className="py-2 pe-3 text-end">
                  {status.kind === 'saved' ? (
                    <Balance balance={status.saved.new_balance} />
                  ) : preview === undefined ? null : (
                    <Balance balance={preview.new_balance} />
                  )}
                </td>
                <td className="py-2 pe-3">
                  {status.kind === 'saved' ? (
                    <span className="text-money-in">
                      <Localized id="vouchers-state-saved" />{' '}
                      <Code>
                        <Localized
                          id="vouchers-number-advance"
                          vars={{ number: String(status.saved.number) }}
                        />
                      </Code>
                    </span>
                  ) : status.kind === 'refused' ? (
                    <span role="alert" className="flex flex-col gap-1 text-due">
                      {status.error.errors.map((item, index) => (
                        <ErrorMessage key={`${item.code}-${String(index)}`} item={item}>
                          <span />
                        </ErrorMessage>
                      ))}
                    </span>
                  ) : (
                    <Localized id="vouchers-state-waiting">
                      <span className="text-ink-muted" />
                    </Localized>
                  )}
                </td>
                <td className="py-2 text-end">
                  {status.kind === 'saved' ? null : (
                    <Button
                      disabled={disabled}
                      onClick={() => {
                        onRemove(voucher.id);
                      }}
                    >
                      <Trash2 aria-hidden="true" className="size-icon-sm" />
                      <span className="sr-only">
                        <Localized id="vouchers-remove" />
                      </span>
                    </Button>
                  )}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
      {total === null ? null : (
        <p className="flex items-baseline gap-2 type-body font-semibold">
          <Localized id="vouchers-list-total" />
          <Num paisa={total} />
        </p>
      )}
    </Card>
  );
}
