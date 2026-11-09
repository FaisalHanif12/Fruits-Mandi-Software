// The voucher screen's page (route /bikri/voucher; PRD FR-7.67, FR-15.10): it reads what the address
// asks for and shows the right thing.
//
// - `?type=receipt|payment|advance`: a new voucher of that type.
// - `?id=12`: a saved voucher. While its day is open it opens in the form for an edit; otherwise
//   (cancelled, or its day closed) it opens read only, with the reason it cannot be changed.
//
// Four states: loading while today or the voucher arrives, an error with a retry, and the screen.

import { Localized } from '@fluent/react';
import { useQueryClient } from '@tanstack/react-query';
import { useNavigate, useRouter, useSearch } from '@tanstack/react-router';
import { useMemo, useState } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import { uiErrorOf } from '../../../shared/api';
import { useGrouping } from '../../../shared/format';
import { SHORTCUTS, useEscape } from '../../../shared/keyboard';
import { OpenOnItsScreen, useFixDayFlow } from '../../../shared/edit';
import { useAccount } from '../../../shared/accounts';
import {
  Button,
  EntryActions,
  ErrorState,
  LoadingState,
  type PickerValue,
} from '../../../shared/ui';
import { CancelVoucherDialog } from '../components/CancelVoucherDialog';
import { SavedVoucherView } from '../components/SavedVoucherView';
import { draftOfRow, fromSideOf, newDraft } from '../draft';
import { VOUCHERS_KEY, useVoucher } from '../hooks';
import { parseVoucherSearch, screenTypeOf, type ScreenType } from '../screen-type';
import { useEditSeed } from '../useEditSeed';
import type { VoucherRow } from '../types';
import { VoucherEditor } from './VoucherEditor';

/**
 * The two buttons under a voucher that cannot be edited yet: of a closed day (Edit reopens the days
 * first, PRD FR-21.4) or locked for another reason, which the buttons tell in words (PRD FR-7.70).
 */
function ClosedEntryActions({ row }: { row: VoucherRow }) {
  const router = useRouter();
  const queryClient = useQueryClient();
  const fixFlow = useFixDayFlow();
  const [cancelRow, setCancelRow] = useState<VoucherRow | null>(null);
  const type = screenTypeOf(row.voucher_type);
  return (
    <>
      <EntryActions
        flags={row}
        busy={fixFlow.busy}
        onEdit={() => {
          fixFlow.start({
            date: row.entry_date,
            ownerHref: router.buildLocation({
              to: '/bikri/voucher',
              search: type === null ? {} : { type, id: row.id },
            }).href,
            // The row is read again; with its day open it comes back ready to edit.
            then: () => {
              void queryClient.invalidateQueries({ queryKey: VOUCHERS_KEY });
            },
          });
        }}
        onCancel={() => {
          setCancelRow(row);
        }}
      />
      {fixFlow.dialog}
      <CancelVoucherDialog
        row={cancelRow}
        onClose={() => {
          setCancelRow(null);
        }}
        onCancelled={() => {
          setCancelRow(null);
        }}
      />
    </>
  );
}

/** A saved voucher: the form for an edit, or the read-only view. */
function SavedVoucher({ row, today }: { row: VoucherRow; today: string }) {
  const navigate = useNavigate();
  const grouping = useGrouping();
  const type = screenTypeOf(row.voucher_type);
  const seed = useEditSeed(row.can_edit && !row.needs_reopen ? row : null);
  const initial = useMemo(
    () =>
      type === null || seed.data === undefined ? null : draftOfRow(row, type, seed.data, grouping),
    [row, type, seed.data, grouping],
  );
  useEscape('leave', () => {
    void navigate({ to: '/bikri/voucher', search: type === null ? {} : { type } });
  });

  // A voucher of another kind (an expense, a loan, a khata entry, capital, a journal) opens on its
  // own screen (plan 44).
  if (type === null) return <OpenOnItsScreen row={row} />;
  if (!row.can_edit || row.needs_reopen) {
    return (
      <div className="flex max-w-page flex-col gap-4">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <Localized id={`vouchers-${type}-title`}>
            <h1 className="type-title" />
          </Localized>
          <Button
            variant="primary"
            shortcut={SHORTCUTS.newEntry}
            onClick={() => {
              void navigate({ to: '/bikri/voucher', search: { type } });
            }}
          >
            <Localized id="vouchers-new" />
          </Button>
        </div>
        <SavedVoucherView row={row} />
        <ClosedEntryActions row={row} />
      </div>
    );
  }
  if (seed.isError) {
    return (
      <ErrorState
        title={<Localized id="vouchers-load-failed" />}
        error={uiErrorOf(seed.error)}
        onRetry={() => {
          void seed.refetch();
        }}
      />
    );
  }
  if (initial === null) return <LoadingState />;
  // A new key for each saved voucher, so the form starts from it and not from the one before.
  return (
    <VoucherEditor
      key={`${String(row.id)}-${String(row.version)}`}
      today={today}
      initial={initial}
      editing={row}
      urlType={type}
    />
  );
}

/**
 * A new voucher that starts with the account an address names (plan 49, PRD FR-7.79): the account is in
 * the picker with its card, and every amount stays empty. Nothing is saved. When the account cannot be
 * read the voucher starts empty, as it does with no party.
 */
function NewVoucherForParty({
  party,
  type,
  today,
}: {
  party: PickerValue;
  type: ScreenType;
  today: string;
}) {
  const account = useAccount(party.id);
  if (account.isPending) return <LoadingState />;
  const blank = newDraft(type, today);
  const accountType = account.data?.account_type ?? null;
  const initial =
    accountType === null
      ? blank
      : {
          ...blank,
          account: party,
          accountType,
          from: type === 'receipt' ? fromSideOf(accountType) : blank.from,
        };
  return (
    <VoucherEditor
      key={`new-${String(party.id)}`}
      today={today}
      initial={initial}
      editing={null}
      urlType={type}
    />
  );
}

export function VoucherPage() {
  const day = useDayStatus();
  // The address is read with the same function the router checks it with (screen-type.ts).
  const rawSearch: Record<string, unknown> = useSearch({ strict: false });
  const search = parseVoucherSearch(rawSearch);
  const saved = useVoucher(search.id ?? null);

  if (day.isPending || (search.id !== undefined && saved.isPending)) return <LoadingState />;
  if (day.isError) {
    return (
      <ErrorState
        title={<Localized id="vouchers-load-failed" />}
        error={uiErrorOf(day.error)}
        onRetry={() => {
          void day.refetch();
        }}
      />
    );
  }
  if (search.id !== undefined) {
    if (saved.isError) {
      return (
        <ErrorState
          title={<Localized id="vouchers-load-failed" />}
          error={uiErrorOf(saved.error)}
          onRetry={() => {
            void saved.refetch();
          }}
        />
      );
    }
    if (saved.data !== undefined) return <SavedVoucher row={saved.data} today={day.data.today} />;
  }
  const type = search.type ?? 'receipt';
  if (search.party !== undefined) {
    return <NewVoucherForParty party={search.party} type={type} today={day.data.today} />;
  }
  return (
    <VoucherEditor
      key="new"
      today={day.data.today}
      initial={newDraft(type, day.data.today)}
      editing={null}
      urlType={type}
    />
  );
}
