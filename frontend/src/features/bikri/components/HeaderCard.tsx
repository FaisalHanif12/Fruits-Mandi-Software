// The header of a بکری (PRD 12.2): date, bill date, truck, زمیندار, اڈا, total freight, and behind
// "مزید" the book, store, driver and note. All as text in the draft above the screen (F39).
//
// - The truck number suggests numbers used before. Taking one (or Enter on a known number) fills
//   the اڈا, the زمیندار and the driver from that truck's last بکری, as visible values the user can
//   change (PRD FR-7.49). If the truck already has a بکری on this day, a notice says so with two
//   buttons (FR-7.50): کھولیں, wired in plan 13, and نئی بکری جاری رکھیں.
// - "تقسیم" splits the total freight over the lots by their نگ. The backend does it (PRD C17); the
//   screen puts each part in the freight cell of its lot.
// - A زمیندار becomes the seller of the first lot (FR-12.2); the lots grid does that.

import { Localized, useLocalization } from '@fluent/react';
import { useQueryClient } from '@tanstack/react-query';
import { ChevronDown, ChevronUp, TriangleAlert } from 'lucide-react';
import { useState } from 'react';
import type { UiError } from '../../../bindings/UiError';
import { accountsGet, pickedOf } from '../../../shared/accounts';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate } from '../../../shared/format';
import { findMaster, type SearchHit } from '../../../shared/search';
import type { PickedRecord } from '../../../shared/grid';
import { useFormNavigation } from '../../../shared/keyboard';
import {
  AccountPicker,
  Button,
  Card,
  DateInput,
  ErrorNotice,
  Field,
  FormGrid,
  FormSpan,
  FreeTextInput,
  MoneyInput,
  Num,
  Picker,
  SuggestInput,
  TextInput,
  fieldError,
} from '../../../shared/ui';
import { assistTruckMemory } from '../api';
import { useBikriDraft, type HeaderState } from '../draft-store';
import { useSplitFreight } from '../hooks/useSplitFreight';
import { useTruckMemory } from '../hooks/useTruckMemory';

interface HeaderCardProps {
  /** The problems of the whole draft, so each field shows the words of its own. */
  error: UiError | null;
  /** Opens the saved بکری of the same truck (plan 13). Without it the button is dimmed. */
  onOpenBikri?: ((number: number) => void) | undefined;
  /** The Enter on the last header field: the screen moves on to the lots. */
  onDone: () => void;
  /**
   * A posting open for edit (plan 16): the day of the posting is kept (PRD FR-12.19), and the
   * same-truck notice is left out, because the sale it would point at is this one.
   */
  editing?: boolean;
}

/** The fields behind "More" that the backend can point at, so a problem on one opens the part. */
const MORE_FIELDS = ['header.bill_date', 'header.total_freight', 'header.driver_phone'] as const;

function pickedOrNull(hit: SearchHit | null): PickedRecord | null {
  return hit === null ? null : pickedOf(hit);
}

export function HeaderCard({ error, onOpenBikri, onDone, editing = false }: HeaderCardProps) {
  const { l10n } = useLocalization();
  const stores = useBikriDraft();
  const queryClient = useQueryClient();
  const header = stores.header();
  const [more, setMore] = useState(false);
  // The same-truck notice is dismissed for this truck and day only.
  const [continued, setContinued] = useState<string | null>(null);
  const [filledFrom, setFilledFrom] = useState<{ number: number; date: string } | null>(null);
  const memory = useTruckMemory(header.truckNo, header.entryDate);
  const split = useSplitFreight();
  const navigation = useFormNavigation(onDone);
  const patch = (change: Partial<HeaderState>) => {
    stores.header.getState().patch(change);
  };

  const truckKey = `${header.truckNo.trim()}|${header.entryDate}`;
  const sameDay = memory.data?.same_day ?? null;
  const notice = !editing && sameDay !== null && continued !== truckKey ? sameDay : null;
  const last = editing ? null : (memory.data?.last ?? null);

  // The truck was taken: fill what the last بکری of this truck knows, into empty fields only.
  const fillFromTruck = async (text: string) => {
    const truck = text.trim();
    if (truck === '') return;
    try {
      const answer = await queryClient.query({
        queryKey: ['bikri', 'truck', 'commit', truck, header.entryDate],
        queryFn: () => assistTruckMemory(truck, header.entryDate),
        staleTime: 0,
      });
      const memoryOfTruck = answer.last;
      if (memoryOfTruck === null) return;
      const filled: Partial<HeaderState> = {};
      const now = stores.header.getState();
      if (now.truckNo.trim() !== truck) return;
      if (memoryOfTruck.adda_id !== null && now.adda === null) {
        filled.adda = pickedOrNull(await findMaster('adda', memoryOfTruck.adda_id));
      }
      if (memoryOfTruck.zamindar_account_id !== null && now.zamindar === null) {
        const account = await accountsGet(memoryOfTruck.zamindar_account_id);
        filled.zamindar = pickedOf(account);
      }
      if (memoryOfTruck.driver_name !== null && now.driverName === '') {
        filled.driverName = memoryOfTruck.driver_name;
      }
      if (memoryOfTruck.driver_phone !== null && now.driverPhone === '') {
        filled.driverPhone = memoryOfTruck.driver_phone;
      }
      patch(filled);
      setFilledFrom({ number: memoryOfTruck.bikri_number, date: memoryOfTruck.date });
    } catch {
      // The memory is a help. If it cannot be read, the user types the fields himself.
    }
  };

  const splitFreight = () => {
    split.mutate({ stores, totalFreight: header.totalFreight });
  };

  // The fields behind "More" that hold something, so a value is never hidden without a sign
  // (plan 38, sale book): the button shows how many. A problem on one of them opens the part.
  const moreValues =
    [
      header.billDate,
      header.totalFreight,
      header.book,
      header.driverName,
      header.driverPhone,
      header.note,
    ].filter((value) => value.trim() !== '').length + (header.store === null ? 0 : 1);
  const moreProblem = MORE_FIELDS.some((field) => fieldError(error, field) !== undefined);
  const open = more || moreProblem;

  return (
    <Card compact>
      <form
        {...navigation}
        onSubmit={(event) => {
          event.preventDefault();
        }}
        className="@container flex flex-col gap-2"
      >
        {/* The fields typed on every بکری are one row: date, truck, زمیندار, اڈا. The rest of the
            header is behind the button at its end (plan 38, sale book). */}
        <div className="flex items-end gap-x-3">
          <div className="w-date shrink-0">
            <Field
              label={<Localized id="bikri-header-date" />}
              error={fieldError(error, 'header.entry_date')}
              readOnly={editing}
            >
              {(control) => (
                <DateInput
                  {...control}
                  value={header.entryDate}
                  onChange={(entryDate) => {
                    patch({ entryDate });
                  }}
                />
              )}
            </Field>
          </div>
          <div className="w-date shrink-0">
            <Field
              label={<Localized id="bikri-header-truck" />}
              error={fieldError(error, 'header.truck_no')}
            >
              {(control) => (
                <SuggestInput
                  {...control}
                  value={header.truckNo}
                  onChange={(truckNo) => {
                    patch({ truckNo });
                  }}
                  suggestions={(memory.data?.suggestions ?? []).map((suggestion) => ({
                    value: suggestion.truck_no,
                    note: isoToDisplayDate(suggestion.date),
                  }))}
                  onCommit={(text) => {
                    // An entry open for edit is never filled from the truck's last sale.
                    if (!editing) void fillFromTruck(text);
                  }}
                />
              )}
            </Field>
          </div>
          <div className="min-w-0 flex-1 basis-mid">
            <Field label={<Localized id="glossary-zamindar" />}>
              {(control) => (
                <AccountPicker
                  {...control}
                  scope="seller"
                  showCard={false}
                  value={header.zamindar}
                  onChange={(hit) => {
                    patch({
                      zamindar: hit === null ? null : pickedOf(hit),
                    });
                  }}
                />
              )}
            </Field>
          </div>
          <div className="min-w-0 flex-1 basis-mid">
            <Field label={<Localized id="glossary-adda" />}>
              {(control) => (
                <Picker
                  {...control}
                  scope="adda"
                  value={header.adda}
                  onChange={(hit) => {
                    patch({ adda: pickedOrNull(hit) });
                  }}
                />
              )}
            </Field>
          </div>
          <Button
            accessibleName={l10n.getString(open ? 'bikri-header-less' : 'bikri-header-more')}
            onClick={() => {
              setMore(!open);
            }}
          >
            {open ? (
              <ChevronUp aria-hidden="true" className="size-icon-md" />
            ) : (
              <ChevronDown aria-hidden="true" className="size-icon-md" />
            )}
            <span className="sr-only @row-roomy:not-sr-only">
              <Localized id={open ? 'bikri-header-less' : 'bikri-header-more'} />
            </span>
            {moreValues > 0 && !open ? (
              <span className="rounded-2 bg-primary-soft px-2 type-label text-primary">
                <Num>{String(moreValues)}</Num>
              </span>
            ) : null}
          </Button>
        </div>
        {open ? (
          <FormGrid columns={4} alignEnd>
            <Field
              label={<Localized id="bikri-header-bill-date" />}
              error={fieldError(error, 'header.bill_date')}
            >
              {(control) => (
                <DateInput
                  {...control}
                  value={header.billDate}
                  placeholder={header.entryDate}
                  onChange={(billDate) => {
                    patch({ billDate });
                  }}
                />
              )}
            </Field>
            <FormSpan span="two">
              <Field
                label={<Localized id="bikri-header-total-freight" />}
                error={fieldError(error, 'header.total_freight')}
              >
                {(control) => (
                  <div className="flex items-start gap-2">
                    <MoneyInput
                      {...control}
                      value={header.totalFreight}
                      onChange={(totalFreight) => {
                        patch({ totalFreight });
                      }}
                    />
                    <Button
                      disabled={header.totalFreight.trim() === ''}
                      busy={split.isPending}
                      onClick={splitFreight}
                    >
                      <Localized id="bikri-header-split-freight" />
                    </Button>
                  </div>
                )}
              </Field>
            </FormSpan>
            <Field label={<Localized id="bikri-header-store" />}>
              {(control) => (
                <Picker
                  {...control}
                  scope="store"
                  value={header.store}
                  onChange={(hit) => {
                    patch({ store: pickedOrNull(hit) });
                  }}
                />
              )}
            </Field>
            <Field label={<Localized id="bikri-header-book" />}>
              {(control) => (
                <TextInput
                  {...control}
                  value={header.book}
                  onChange={(book) => {
                    patch({ book });
                  }}
                />
              )}
            </Field>
            <Field label={<Localized id="bikri-header-driver" />}>
              {(control) => (
                <FreeTextInput
                  {...control}
                  value={header.driverName}
                  onChange={(driverName) => {
                    patch({ driverName });
                  }}
                />
              )}
            </Field>
            <Field
              label={<Localized id="bikri-header-driver-phone" />}
              error={fieldError(error, 'header.driver_phone')}
            >
              {(control) => (
                <TextInput
                  {...control}
                  value={header.driverPhone}
                  onChange={(driverPhone) => {
                    patch({ driverPhone });
                  }}
                />
              )}
            </Field>
            <FormSpan span="two">
              <Field label={<Localized id="bikri-header-note" />}>
                {(control) => (
                  <FreeTextInput
                    {...control}
                    value={header.note}
                    onChange={(note) => {
                      patch({ note });
                    }}
                  />
                )}
              </Field>
            </FormSpan>
          </FormGrid>
        ) : null}
        {filledFrom !== null ? (
          <Localized
            id="bikri-header-filled-from"
            vars={{ number: String(filledFrom.number), date: isoToDisplayDate(filledFrom.date) }}
          >
            <p className="type-body text-ink-muted" />
          </Localized>
        ) : last === null || notice !== null ? null : (
          <Localized
            id="bikri-truck-last"
            vars={{ number: String(last.bikri_number), date: isoToDisplayDate(last.date) }}
          >
            <p className="type-body text-ink-muted" />
          </Localized>
        )}
        {split.isError ? <ErrorNotice error={uiErrorOf(split.error)} /> : null}
      </form>
      {notice === null ? null : (
        <div
          role="status"
          className="flex items-center gap-3 rounded-2 border border-warn-border bg-warn-soft px-3 py-2 text-warn"
        >
          <TriangleAlert aria-hidden="true" className="size-icon-md shrink-0" />
          <Localized
            id="bikri-truck-same-day"
            vars={{ number: String(notice.bikri_number), date: header.entryDate }}
          >
            <p className="min-w-0 flex-1 type-body" />
          </Localized>
          <div className="flex shrink-0 gap-2">
            <Button
              disabled={onOpenBikri === undefined}
              onClick={() => {
                onOpenBikri?.(notice.bikri_number);
              }}
            >
              <Localized id="bikri-truck-open" />
            </Button>
            <Button
              onClick={() => {
                setContinued(truckKey);
              }}
            >
              <Localized id="bikri-truck-continue" />
            </Button>
          </div>
        </div>
      )}
    </Card>
  );
}
