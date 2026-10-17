// The voucher screen's page (route /bikri/voucher; PRD FR-7.67, FR-15.10): it reads what the address
// asks for and shows the right thing.
//
// - `?type=receipt|payment|advance`: a new voucher of that type.
// - `?id=12`: a saved voucher. While its day is open it opens in the form for an edit; otherwise
//   (cancelled, or its day closed) it opens read only, with the reason it cannot be changed.
//
// Four states: loading while today or the voucher arrives, an error with a retry, and the screen.

import { Localized } from '@fluent/react';
import { useNavigate, useSearch } from '@tanstack/react-router';
import { FileQuestion } from 'lucide-react';
import { useMemo } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import { uiErrorOf } from '../../../shared/api';
import { useGrouping } from '../../../shared/format';
import { useLanguage } from '../../../shared/i18n';
import { SHORTCUTS, useEscape } from '../../../shared/keyboard';
import { Button, EmptyState, ErrorState, LoadingState } from '../../../shared/ui';
import { SavedVoucherView } from '../components/SavedVoucherView';
import { draftOfRow, newDraft } from '../draft';
import { useVoucher } from '../hooks';
import { parseVoucherSearch, screenTypeOf } from '../screen-type';
import { useEditSeed } from '../useEditSeed';
import type { VoucherRow } from '../types';
import { VoucherEditor } from './VoucherEditor';

/** A saved voucher: the form for an edit, or the read-only view. */
function SavedVoucher({ row, today }: { row: VoucherRow; today: string }) {
  const navigate = useNavigate();
  const language = useLanguage();
  const grouping = useGrouping();
  const type = screenTypeOf(row.voucher_type);
  const seed = useEditSeed(row.editable ? row : null);
  const initial = useMemo(
    () =>
      type === null || seed.data === undefined
        ? null
        : draftOfRow(row, type, seed.data, grouping, language),
    [row, type, seed.data, grouping, language],
  );
  useEscape('leave', () => {
    void navigate({ to: '/bikri/voucher', search: type === null ? {} : { type } });
  });

  if (type === null) {
    return (
      <EmptyState
        icon={FileQuestion}
        title={<Localized id="vouchers-other-screen" />}
        help={<Localized id="vouchers-other-screen-help" />}
        action={
          <Button
            variant="primary"
            onClick={() => {
              void navigate({ to: '/rokar' });
            }}
          >
            <Localized id="vouchers-other-screen-open" />
          </Button>
        }
      />
    );
  }
  if (!row.editable) {
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
