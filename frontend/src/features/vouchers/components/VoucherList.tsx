// The running list of vouchers waiting to be saved together (PRD FR-15.1): the seller, the amount,
// the balance after, and the state of each (waiting, saved with its number, or refused with what
// to do). The total is the backend's. Each voucher is its own posting event, so a refused one
// stays here with its message while the others are saved. Four states of the whole screen live in
// the page; this list shows nothing when it is empty.

import { Localized, useLocalization } from '@fluent/react';
import { Trash2 } from 'lucide-react';
import { ErrorMessage } from '../../../shared/ui/ErrorMessage';
import { displayName, useLanguage } from '../../../shared/i18n';
import { Balance, Button, Card, Code, Num, Table, type TableColumn } from '../../../shared/ui';
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
  const { l10n } = useLocalization();
  if (vouchers.length === 0) return null;
  const columns: TableColumn<ListedVoucher>[] = [
    {
      id: 'seller',
      header: <Localized id="vouchers-col-seller" />,
      cell: (voucher) =>
        voucher.draft.seller === null ? null : (
          <span className="flex flex-wrap items-baseline gap-x-2">
            <Code>{voucher.draft.seller.code}</Code>
            <span>{displayName(voucher.draft.seller, language)}</span>
          </span>
        ),
      total: total === null ? undefined : <Localized id="vouchers-list-total" />,
    },
    {
      id: 'amount',
      header: <Localized id="vouchers-col-amount" />,
      kind: 'number',
      cell: (voucher) => {
        const preview = previews.get(voucher.id);
        return preview === undefined ? null : <Num paisa={preview.amount_paisa} />;
      },
      // The total of the list is the backend's, and sits under the amounts it adds up.
      total: total === null ? undefined : <Num paisa={total} />,
    },
    {
      id: 'balance',
      header: <Localized id="vouchers-col-new-balance" />,
      kind: 'number',
      cell: (voucher) => {
        const { status } = voucher;
        const preview = previews.get(voucher.id);
        if (status.kind === 'saved') return <Balance balance={status.saved.new_balance} />;
        return preview === undefined ? null : <Balance balance={preview.new_balance} />;
      },
      total: total === null ? undefined : '',
    },
    {
      id: 'state',
      header: <Localized id="vouchers-col-state" />,
      cell: ({ status }) => {
        if (status.kind === 'saved') {
          return (
            <span className="text-money-in">
              <Localized id="vouchers-state-saved" />{' '}
              <Code>
                <Localized
                  id="vouchers-number-advance"
                  vars={{ number: String(status.saved.number) }}
                />
              </Code>
            </span>
          );
        }
        if (status.kind === 'refused') {
          return (
            <span role="alert" className="flex flex-col gap-1 text-due">
              {status.error.errors.map((item, index) => (
                <ErrorMessage key={`${item.code}-${String(index)}`} item={item}>
                  <span />
                </ErrorMessage>
              ))}
            </span>
          );
        }
        return (
          <Localized id="vouchers-state-waiting">
            <span className="text-ink-muted" />
          </Localized>
        );
      },
      total: total === null ? undefined : '',
    },
    {
      id: 'remove',
      header: <Localized id="vouchers-remove" />,
      kind: 'action',
      cell: (voucher) =>
        voucher.status.kind === 'saved' ? null : (
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
        ),
      total: total === null ? undefined : '',
    },
  ];
  return (
    <Card title={<Localized id="vouchers-list-title" />}>
      <Table
        label={l10n.getString('vouchers-list-title')}
        columns={columns}
        rows={vouchers}
        rowKey={(voucher) => voucher.id}
      />
    </Card>
  );
}
