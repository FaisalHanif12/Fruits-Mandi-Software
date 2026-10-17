// The vouchers waiting to be saved together (PRD FR-15.1): the type, the account, the amount and
// the balance after as the backend works them out, and the state of each (waiting, refused with the
// reason, or not saved because the user went back at the cash question). F10 saves all of them, one
// by one, each its own posting event. A voucher that is saved leaves the list: it is in the list of
// the day under the form. Nothing here adds money; the amounts are the backend's, from each
// voucher's preview. The list is shown only while it has a voucher in it.

import { Localized, useLocalization } from '@fluent/react';
import { Trash2 } from 'lucide-react';
import { displayName, useLanguage } from '../../../shared/i18n';
import { SHORTCUTS } from '../../../shared/keyboard';
import { Balance, Button, Card, Code, Num, Table, type TableColumn } from '../../../shared/ui';
import { ErrorMessage } from '../../../shared/ui/ErrorMessage';
import type { ListedVoucher } from '../draft';
import type { VoucherPreview } from '../types';

interface WaitingListProps {
  vouchers: readonly ListedVoucher[];
  /** The backend's view of each voucher by its id, for the amount and the balance after. */
  previews: ReadonlyMap<string, VoucherPreview>;
  onRemove: (id: string) => void;
  /** A run is saving: nothing can be removed now. */
  disabled: boolean;
}

export function WaitingList({ vouchers, previews, onRemove, disabled }: WaitingListProps) {
  const language = useLanguage();
  const { l10n } = useLocalization();
  if (vouchers.length === 0) return null;
  const columns: TableColumn<ListedVoucher>[] = [
    {
      id: 'type',
      header: <Localized id="vouchers-col-type" />,
      cell: (voucher) => <Localized id={`vouchers-type-${voucher.draft.type}`} />,
    },
    {
      id: 'account',
      header: <Localized id="vouchers-col-account" />,
      cell: ({ draft }) => {
        const { account, person } = draft;
        if (account === null) return null;
        return (
          <span className="flex flex-wrap items-baseline gap-x-2">
            <Code>{account.code}</Code>
            <span>{displayName(person ?? account, language)}</span>
          </span>
        );
      },
    },
    {
      id: 'amount',
      header: <Localized id="vouchers-col-amount" />,
      kind: 'number',
      cell: (voucher) => {
        const preview = previews.get(voucher.id);
        return preview === undefined ? null : <Num paisa={preview.amount_paisa} />;
      },
    },
    {
      id: 'balance',
      header: <Localized id="vouchers-new-balance" />,
      kind: 'number',
      cell: (voucher) => {
        const preview = previews.get(voucher.id);
        return preview === undefined ? null : <Balance balance={preview.new_balance} />;
      },
    },
    {
      id: 'state',
      header: <Localized id="vouchers-col-state" />,
      cell: ({ status }) => {
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
          <Localized
            id={status.kind === 'declined' ? 'vouchers-state-declined' : 'vouchers-state-waiting'}
          >
            <span className={status.kind === 'declined' ? 'text-warn' : 'text-ink-muted'} />
          </Localized>
        );
      },
    },
    {
      id: 'remove',
      header: <Localized id="vouchers-remove" />,
      kind: 'action',
      cell: (voucher) => (
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
    },
  ];
  return (
    <Card compact title={<Localized id="vouchers-waiting-title" />}>
      <Localized id="vouchers-waiting-help" vars={{ key: SHORTCUTS.save.label }}>
        <p className="type-body text-ink-muted" />
      </Localized>
      <Table
        label={l10n.getString('vouchers-waiting-title')}
        columns={columns}
        rows={vouchers}
        rowKey={(voucher) => voucher.id}
      />
    </Card>
  );
}
