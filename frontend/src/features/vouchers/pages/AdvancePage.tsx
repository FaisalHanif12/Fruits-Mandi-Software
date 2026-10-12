// پیشگی کھاتہ: giving an advance to a seller (route /bikri/advance; PRD 15.1, 15.4, FR-7.60,
// FR-15.1). One voucher form with the seller's party card and advances, the balance before and
// after from the backend, and a running list so several vouchers of one sitting can be saved
// together. Each voucher is its own posting event: one that is refused stays in the list with its
// message while the others are saved.
//
// - Keys: F10 or Ctrl+S saves (a lone voucher straight from the form, or the whole list), F2 adds
//   the voucher on the form to the list and starts the next one, Esc goes back (asking first).
// - Everything typed is text in this screen's state, so a language switch loses nothing (F39).
// - The frontend works out no money: the amount goes as typed; the balances, the amounts and the
//   total of the list come from the backend (the preview is the same function as the save).
// - Four states: loading while today arrives, error with a retry, the form, and the list.

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { useMemo, useState } from 'react';
import { LeaveGuard } from '../../../app/LeaveGuard';
import { useDayStatus } from '../../../app/use-day-status';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { SHORTCUTS, useEscape, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import { useFocusFirstField } from '../../../shared/hooks';
import { Button, ErrorNotice, ErrorState, LoadingState, showToast } from '../../../shared/ui';
import { AdvanceForm } from '../components/AdvanceForm';
import { VoucherList } from '../components/VoucherList';
import {
  isFilled,
  listed,
  newAdvanceDraft,
  nextAdvanceDraft,
  toSave,
  type ListedVoucher,
} from '../draft';
import { useAdvancePreview, useSaveAdvance } from '../hooks';
import type { AdvanceDraft, AdvancePreviewRow, VoucherSaved } from '../types';

/** The fields the form shows next to their inputs, so the notice never tells a problem twice. */
const FIELD_PATHS = [
  'date',
  'seller_account_id',
  'amount',
  'mode',
  'bank_account_id',
  'cheque_no',
  'cheque_date',
  'expected_goods',
  'narration',
];

/** The problems of a preview row, shaped like a refused save so the fields can show them. */
function errorOfPreview(row: AdvancePreviewRow | null): UiError | null {
  if (row === null || row.errors.length === 0) return null;
  return {
    ok: false,
    errors: row.errors.map((error) => ({ code: error.code, field: error.field, values: {} })),
    ref: '',
  };
}

function AdvanceEditor({ today }: { today: string }) {
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  const [draft, setDraft] = useState<AdvanceDraft>(() => newAdvanceDraft(today));
  const [list, setList] = useState<ListedVoucher[]>([]);
  // What the backend refused when the voucher on the form was saved; cleared by the next edit.
  const [refused, setRefused] = useState<UiError | null>(null);
  const [saving, setSaving] = useState(false);
  const save = useSaveAdvance();
  const formRef = useFocusFirstField<HTMLFormElement>();

  const pending = useMemo(() => toSave(list), [list]);
  const filled = isFilled(draft);
  const drafts = useMemo(
    () => [...pending.map((voucher) => voucher.draft), ...(filled ? [draft] : [])],
    [pending, draft, filled],
  );
  const { settled, query } = useAdvancePreview(drafts);

  // The backend's view of each voucher, found by the identity of its draft. While an older answer
  // is kept for a newer question, the rows no longer line up, so only the total is shown.
  const rows = query.data !== undefined && !query.isPlaceholderData ? query.data.rows : null;
  const previewOf = (candidate: AdvanceDraft): AdvancePreviewRow | null => {
    const index = settled.indexOf(candidate);
    return rows === null || index < 0 ? null : (rows[index] ?? null);
  };
  const previews = new Map<string, AdvancePreviewRow>();
  for (const voucher of pending) {
    const row = previewOf(voucher.draft);
    if (row !== null) previews.set(voucher.id, row);
  }
  const formPreview = filled ? previewOf(draft) : null;
  const error = refused ?? errorOfPreview(formPreview);
  const total = query.data !== undefined && drafts.length > 0 ? query.data.total_paisa : null;

  const change = (patch: Partial<AdvanceDraft>) => {
    setRefused(null);
    setDraft((current) => ({ ...current, ...patch }));
  };

  const markSaved = (id: string, saved: VoucherSaved) => {
    setList((current) =>
      current.map((voucher) =>
        voucher.id === id ? { ...voucher, status: { kind: 'saved', saved } } : voucher,
      ),
    );
  };
  const markRefused = (id: string, failure: UiError) => {
    setList((current) =>
      current.map((voucher) =>
        voucher.id === id ? { ...voucher, status: { kind: 'refused', error: failure } } : voucher,
      ),
    );
  };
  const savedToast = (saved: VoucherSaved) => {
    showToast(
      'success',
      l10n.getString('vouchers-saved-advance', {
        number: l10n.getString('vouchers-number-advance', { number: String(saved.number) }),
      }),
    );
  };

  const addToList = () => {
    if (!filled) return;
    setList((current) => [...current, listed(draft)]);
    setDraft(nextAdvanceDraft(draft));
    setRefused(null);
    // The next voucher starts at the seller, the field that always changes.
    formRef.current?.querySelector<HTMLInputElement>('input[role="combobox"]')?.focus();
  };

  // A lone voucher is saved from the form, so a refusal stays on its fields. With vouchers in the
  // list, the form joins the list and each voucher is saved in turn, each its own transaction.
  const submit = async (): Promise<boolean> => {
    if (saving) return false;
    if (pending.length === 0 && !filled) return true;
    setSaving(true);
    try {
      if (pending.length === 0) {
        const saved = await save.mutateAsync(draft);
        setList((current) => [...current, { ...listed(draft), status: { kind: 'saved', saved } }]);
        setDraft(nextAdvanceDraft(draft));
        savedToast(saved);
        return true;
      }
      const joined = filled ? listed(draft) : null;
      if (joined !== null) {
        setList((current) => [...current, joined]);
        setDraft(nextAdvanceDraft(draft));
      }
      const todo = joined === null ? pending : [...pending, joined];
      let refusals = 0;
      let lastSaved: VoucherSaved | null = null;
      for (const voucher of todo) {
        try {
          const saved = await save.mutateAsync(voucher.draft);
          markSaved(voucher.id, saved);
          lastSaved = saved;
        } catch (failure: unknown) {
          refusals += 1;
          markRefused(voucher.id, uiErrorOf(failure));
        }
      }
      if (refusals === 0 && lastSaved !== null) {
        showToast(
          'success',
          todo.length === 1
            ? l10n.getString('vouchers-saved-advance', {
                number: l10n.getString('vouchers-number-advance', {
                  number: String(lastSaved.number),
                }),
              })
            : l10n.getString('vouchers-saved-many', { count: String(todo.length) }),
        );
      } else if (refusals > 0) {
        showToast('error', l10n.getString('vouchers-some-refused', { count: String(refusals) }));
      }
      return refusals === 0;
    } catch (failure: unknown) {
      setRefused(uiErrorOf(failure));
      return false;
    } finally {
      setSaving(false);
    }
  };
  const submitNow = () => {
    void submit();
  };
  const navigation = useFormNavigation(submitNow);
  useShortcut(SHORTCUTS.save, submitNow);
  useShortcut(SHORTCUTS.newEntry, addToList);
  useEscape('leave', () => {
    void navigate({ to: '/bikri' });
  });

  const dirty = pending.length > 0 || filled;
  const canAdd = filled;

  return (
    <div className="flex max-w-page flex-col gap-5">
      <LeaveGuard dirty={dirty} onSave={submit} />
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex flex-col gap-1">
          <Localized id="vouchers-advance-title">
            <h1 className="type-title" />
          </Localized>
          <Localized id="vouchers-advance-help">
            <p className="type-body text-ink-muted" />
          </Localized>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button disabled={!canAdd || saving} shortcut={SHORTCUTS.newEntry} onClick={addToList}>
            <Localized id="vouchers-add-to-list" />
          </Button>
          <Button
            variant="primary"
            shortcut={SHORTCUTS.save}
            busy={saving}
            disabled={!dirty}
            onClick={submitNow}
          >
            <Localized id="layout-save-save" />
          </Button>
        </div>
      </div>
      <form
        ref={formRef}
        {...navigation}
        onSubmit={(event) => {
          event.preventDefault();
        }}
        className="flex flex-col gap-4"
      >
        {error === null ? null : <ErrorNotice error={error} fieldsShown={FIELD_PATHS} />}
        <AdvanceForm draft={draft} onChange={change} error={error} preview={formPreview} />
      </form>
      <VoucherList
        vouchers={list}
        previews={previews}
        total={total}
        disabled={saving}
        onRemove={(id) => {
          setList((current) => current.filter((voucher) => voucher.id !== id));
        }}
      />
    </div>
  );
}

export function AdvancePage() {
  const day = useDayStatus();
  if (day.isPending) return <LoadingState />;
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
  return <AdvanceEditor today={day.data.today} />;
}
