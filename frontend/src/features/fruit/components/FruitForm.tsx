// The fields of the پھل form (PRD 16.7, FR-16.10, FR-16.11, FR-7.53): for whom, the day, where the
// fruit comes from, the lot (or the other agent with an item and packing), the quantity, the rate and
// a note. Everything typed is text; the backend reads it (CLAUDE.md rule 1). A problem the backend
// found shows under the field it points at.
//
// - From our shop the item and packing come from the lot and are shown, not typed; the weight shows
//   only for an item sold by weight. From another shop the item and packing are picked.
// - The rate stays empty. The last rate of the item shows as grey text and as a hint under the field
//   and is never put into it (PRD FR-7.53, Code Standards F43).
// - The order of the fields is the order of the keys: Enter goes to the next field (PRD FR-7.14).

import { Localized, useLocalization } from '@fluent/react';
import { uiErrorOf } from '../../../shared/api';
import type { UiError } from '../../../bindings/UiError';
import { formatPaisa, useGrouping } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import {
  AccountPicker,
  DateInput,
  ErrorState,
  Fact,
  Field,
  FreeTextInput,
  MoneyInput,
  PickerBase,
  Picker,
  QtyInput,
  SegmentedControl,
  Select,
  fieldError,
  FormCard,
  FormSpan,
} from '../../../shared/ui';
import { usePartnerList } from '../../partners';
import { changeSource, chosenLotOf, valueOfRef } from '../draft';
import { LOT_SOURCE, type LotHit } from '../lot-source';
import { FOR_GUESTS, type FruitDraft, type FruitPreview, type FruitSource } from '../types';

interface FruitFormProps {
  draft: FruitDraft;
  /**
   * Changes the draft from the draft as it is NOW, not as this render saw it: two picks that come
   * before the next render (a slow machine) cannot undo each other.
   */
  onChange: (change: (current: FruitDraft) => FruitDraft) => void;
  /** What the backend refused, to show next to the field it points at. */
  error: UiError | null;
  /** The backend's answer for the form as it is, for the last rate and the lot's balance. */
  preview: FruitPreview | undefined;
  /** An entry is being edited: the reason is asked too (PRD FR-16.12). */
  editing: boolean;
}

export function FruitForm({ draft, onChange, error, preview, editing }: FruitFormProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const grouping = useGrouping();
  const partners = usePartnerList();
  const shop = draft.source === 'shop';
  const patch = (next: Partial<FruitDraft>) => {
    onChange((current) => ({ ...current, ...next }));
  };

  // The last rate is grey text only; it is never filled into the field (PRD FR-7.53).
  const hint = preview?.rate_hint ?? null;
  const hintRate = hint === null ? undefined : formatPaisa(hint.rate_paisa, { grouping });
  const hintWords =
    hint === null
      ? undefined
      : l10n.getString(`bikri-rate-hint-${hint.source.replaceAll('_', '-')}`, {
          rate: formatPaisa(hint.rate_paisa, { grouping }),
          date: hint.date,
        });

  const lotRow = draft.lot?.row ?? null;
  const remaining = preview?.numbers?.lot?.balance_before ?? lotRow?.balance ?? null;
  const showWeight = shop ? (lotRow?.sold_by_weight ?? draft.weight.trim() !== '') : true;
  const chosenItem = shop ? (preview?.numbers?.item ?? null) : null;
  const chosenPacking = shop ? (preview?.numbers?.packing ?? null) : null;

  const forField = partners.isError ? (
    <ErrorState
      title={<Localized id="fruit-partners-failed" />}
      error={uiErrorOf(partners.error)}
      onRetry={() => {
        void partners.refetch();
      }}
    />
  ) : (
    <Field
      label={<Localized id="fruit-field-for" />}
      hint={<Localized id="fruit-for-hint" />}
      error={fieldError(error, 'partner_id')}
    >
      {(props) => (
        <Select
          {...props}
          readOnly={props.readOnly === true || partners.isPending}
          value={draft.forWhom === '' ? undefined : draft.forWhom}
          onChange={(forWhom) => {
            patch({ forWhom });
          }}
          options={[
            ...(partners.data ?? [])
              .filter((partner) => partner.is_active)
              .map((partner) => ({
                value: String(partner.id),
                label: displayName(partner, language),
              })),
            { value: FOR_GUESTS, label: l10n.getString('fruit-for-guests') },
          ]}
        />
      )}
    </Field>
  );

  const sourceField = (
    <div className="flex min-w-0 flex-col gap-2">
      <Localized id="fruit-field-source">
        <span className="type-label text-ink" />
      </Localized>
      <SegmentedControl<FruitSource>
        groupLabel={l10n.getString('fruit-field-source')}
        value={draft.source}
        onChange={(source) => {
          onChange((current) => changeSource(current, source));
        }}
        options={[
          { value: 'shop', label: <Localized id="fruit-source-own" /> },
          { value: 'other_shop', label: <Localized id="fruit-source-other" /> },
        ]}
      />
      <Localized id={shop ? 'fruit-source-own-help' : 'fruit-source-other-help'}>
        <p className="type-body text-ink-muted" />
      </Localized>
    </div>
  );

  const lotField = (
    <Field
      label={<Localized id="fruit-field-lot" />}
      hint={<Localized id="fruit-lot-hint" />}
      error={fieldError(error, 'lot_id')}
    >
      {(props) => (
        <PickerBase<LotHit>
          {...props}
          source={LOT_SOURCE}
          value={draft.lot?.value ?? null}
          onChange={(hit) => {
            patch({
              lot: hit === null ? null : chosenLotOf(hit.row),
              // The item and packing follow the lot; a weight typed for another lot goes.
              weight: '',
            });
          }}
          renderEnd={(hit) => (
            <Localized id="fruit-lot-left" vars={{ remaining: String(hit.row.balance) }}>
              <span className="type-body text-ink-muted" />
            </Localized>
          )}
        />
      )}
    </Field>
  );

  const otherFields = (
    <>
      <Field
        label={<Localized id="fruit-field-agent" />}
        hint={<Localized id="fruit-agent-hint" />}
        error={fieldError(error, 'agent_account_id')}
      >
        {(props) => (
          <AccountPicker
            {...props}
            scope="seller"
            value={draft.agent}
            onChange={(hit) => {
              patch({ agent: hit === null ? null : valueOfRef(hit) });
            }}
          />
        )}
      </Field>
      <Field label={<Localized id="glossary-qism-maal" />} error={fieldError(error, 'item_id')}>
        {(props) => (
          <Picker
            {...props}
            scope="item"
            value={draft.item}
            onChange={(hit) => {
              patch({ item: hit === null ? null : valueOfRef(hit) });
            }}
          />
        )}
      </Field>
      <Field label={<Localized id="glossary-packing" />} error={fieldError(error, 'packing_id')}>
        {(props) => (
          <Picker
            {...props}
            scope="packing"
            value={draft.packing}
            onChange={(hit) => {
              patch({ packing: hit === null ? null : valueOfRef(hit) });
            }}
          />
        )}
      </Field>
    </>
  );

  return (
    <FormCard>
      {forField}
      <Field label={<Localized id="fruit-field-date" />} error={fieldError(error, 'date')}>
        {(props) => (
          <DateInput
            {...props}
            value={draft.date}
            onChange={(date) => {
              patch({ date });
            }}
          />
        )}
      </Field>
      <FormSpan>{sourceField}</FormSpan>
      {shop ? lotField : otherFields}
      {shop ? (
        <div className="self-end">
          {chosenItem === null || chosenPacking === null ? null : (
            <dl className="grid grid-cols-2 gap-3">
              <Fact label={<Localized id="glossary-qism-maal" />}>
                {displayName(chosenItem, language)}
              </Fact>
              <Fact label={<Localized id="glossary-packing" />}>
                {displayName(chosenPacking, language)}
              </Fact>
            </dl>
          )}
        </div>
      ) : null}
      <Field
        label={<Localized id="fruit-field-qty" />}
        hint={
          remaining === null ? undefined : (
            <Localized id="fruit-qty-left" vars={{ remaining: String(remaining) }} />
          )
        }
        error={fieldError(error, 'qty')}
      >
        {(props) => (
          <QtyInput
            {...props}
            value={draft.qty}
            onChange={(qty) => {
              patch({ qty });
            }}
          />
        )}
      </Field>
      {showWeight ? (
        <Field label={<Localized id="fruit-field-weight" />} error={fieldError(error, 'weight')}>
          {(props) => (
            <QtyInput
              {...props}
              value={draft.weight}
              onChange={(weight) => {
                patch({ weight });
              }}
            />
          )}
        </Field>
      ) : null}
      <Field
        label={<Localized id="fruit-field-rate" />}
        hint={hintWords === undefined ? undefined : <span>{hintWords}</span>}
        error={fieldError(error, 'rate')}
      >
        {(props) => (
          <MoneyInput
            {...props}
            value={draft.rate}
            {...(draft.rate === '' && hintRate !== undefined ? { placeholder: hintRate } : {})}
            onChange={(rate) => {
              patch({ rate });
            }}
          />
        )}
      </Field>
      <FormSpan>
        <Field label={<Localized id="fruit-field-note" />} error={fieldError(error, 'note')}>
          {(props) => (
            <FreeTextInput
              {...props}
              value={draft.note}
              onChange={(note) => {
                patch({ note });
              }}
            />
          )}
        </Field>
      </FormSpan>
      {editing ? (
        <FormSpan>
          <Field label={<Localized id="fruit-field-reason" />} error={fieldError(error, 'reason')}>
            {(props) => (
              <FreeTextInput
                {...props}
                value={draft.reason}
                onChange={(reason) => {
                  patch({ reason });
                }}
              />
            )}
          </Field>
        </FormSpan>
      ) : null}
    </FormCard>
  );
}
