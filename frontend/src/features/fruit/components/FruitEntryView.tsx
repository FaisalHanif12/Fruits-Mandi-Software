// One saved entry in its saved view, opened by its address (`/rokar/fruit?entry=7`, PRD FR-19.4: the
// register opens a row to see its source). It shows everything the entry holds, as the backend has
// it now: for whom, the day, where the fruit came from (the lot or the agent), the item, the
// quantity, the rate, the value and each partner's part, and says in words when it was cancelled.
// Edit and cancel are the same two buttons as the list's, and work while its day is open. A wrong or
// unknown id says the entry was not found, in the normal words, and the form stays usable.
//
// Four states: loading, the not-found words (or an error with a retry), and the entry. Every
// figure is the backend's (CLAUDE.md rule 1); names go through `displayName` (F38).

import { Localized, useLocalization } from '@fluent/react';
import { Ban, Pencil, X } from 'lucide-react';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { formatGrams, isoToDisplayDate } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import {
  Button,
  Card,
  ErrorNotice,
  ErrorState,
  Fact,
  LoadingState,
  Notice,
  Num,
} from '../../../shared/ui';
import { useFruitEntry } from '../hooks';
import { fruitNumberText } from '../list-columns';
import { lotCode } from '../lot-source';
import type { FruitRow } from '../types';

interface FruitEntryViewProps {
  /** The id in the address, or `'bad'` when the address holds something that is not an id. */
  entry: number | 'bad';
  onEdit: (row: FruitRow) => void;
  onCancel: (row: FruitRow) => void;
  /** Leave the saved view: the address goes back to the plain screen. */
  onClose: () => void;
}

/** What an id that cannot be an id says: the same words as an id the backend does not know. */
const NOT_FOUND: UiError = {
  ok: false,
  errors: [{ code: 'ERR_NOT_FOUND', field: 'id', values: {} }],
  ref: '',
};

function CloseButton({ onClose }: { onClose: () => void }) {
  return (
    <Button onClick={onClose}>
      <X aria-hidden="true" className="size-icon-md" />
      <Localized id="fruit-entry-close" />
    </Button>
  );
}

function NotFound({ error, onClose }: { error: UiError; onClose: () => void }) {
  return (
    <Card compact title={<Localized id="fruit-entry-failed" />}>
      <ErrorNotice error={error} />
      <div>
        <CloseButton onClose={onClose} />
      </div>
    </Card>
  );
}

function Saved({
  row,
  onEdit,
  onCancel,
  onClose,
}: {
  row: FruitRow;
  onEdit: (row: FruitRow) => void;
  onCancel: (row: FruitRow) => void;
  onClose: () => void;
}) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const say = (id: string, vars?: Record<string, string>) => l10n.getString(id, vars);
  const cancelled = row.status === 'cancelled';
  const credited = row.pukhta ?? row.credit_account;
  return (
    <Card
      compact
      title={
        <Localized id="fruit-entry-title" vars={{ number: fruitNumberText(row.number, say) }}>
          <span />
        </Localized>
      }
    >
      {cancelled ? (
        <Notice tone="warn">
          <Localized id="fruit-entry-cancelled" vars={{ reason: row.cancel_reason ?? '' }} />
        </Notice>
      ) : null}
      <dl className="grid grid-cols-2 gap-x-4 gap-y-3 normal:grid-cols-4">
        <Fact label={<Localized id="fruit-field-for" />}>
          {row.partner === null
            ? l10n.getString('fruit-for-guests')
            : displayName(row.partner, language)}
        </Fact>
        <Fact label={<Localized id="fruit-field-date" />}>
          <Num>{isoToDisplayDate(row.entry_date)}</Num>
        </Fact>
        <Fact label={<Localized id="fruit-field-source" />}>
          <Localized id={row.source === 'shop' ? 'fruit-source-own' : 'fruit-source-other'} />
        </Fact>
        <Fact label={<Localized id="fruit-col-from" />}>
          {row.lot === null ? (
            displayName(credited, language)
          ) : (
            <Localized id="fruit-from-lot" vars={{ lot: lotCode(row.lot) }} />
          )}
        </Fact>
        <Fact label={<Localized id="glossary-qism-maal" />}>{displayName(row.item, language)}</Fact>
        <Fact label={<Localized id="glossary-packing" />}>
          {displayName(row.packing, language)}
        </Fact>
        <Fact label={<Localized id="fruit-field-qty" />}>
          <Num>{String(row.qty)}</Num>
        </Fact>
        {row.weight_grams === null ? null : (
          <Fact label={<Localized id="fruit-field-weight" />}>
            <Num>{formatGrams(row.weight_grams)}</Num>
          </Fact>
        )}
        <Fact label={<Localized id="fruit-field-rate" />}>
          <Num paisa={row.rate_paisa} />
        </Fact>
        <Fact label={<Localized id="fruit-numbers-value" />}>
          <Num paisa={row.value_paisa} />
        </Fact>
        <Fact label={<Localized id="fruit-numbers-credit" />}>
          {displayName(credited, language)}
        </Fact>
        {row.note === null || row.note === '' ? null : (
          <Fact label={<Localized id="fruit-field-note" />}>{row.note}</Fact>
        )}
      </dl>
      {row.parts.length === 0 ? null : (
        <div className="flex flex-col gap-1">
          <Localized id="fruit-entry-parts">
            <h3 className="type-label text-ink-muted" />
          </Localized>
          <ul className="flex flex-col">
            {row.parts.map((part) => (
              <li
                key={part.partner.id}
                className="flex items-baseline justify-between gap-3 border-b border-border py-1 type-body text-ink"
              >
                <span>{displayName(part.partner, language)}</span>
                <Num paisa={part.amount_paisa} />
              </li>
            ))}
          </ul>
        </div>
      )}
      <div className="flex flex-wrap items-center gap-2">
        <Button
          disabled={!row.can_change}
          onClick={() => {
            onEdit(row);
          }}
        >
          <Pencil aria-hidden="true" className="size-icon-md" />
          <Localized id="fruit-edit" />
        </Button>
        <Button
          variant="danger"
          disabled={!row.can_change}
          onClick={() => {
            onCancel(row);
          }}
        >
          <Ban aria-hidden="true" className="size-icon-md" />
          <Localized id="fruit-cancel" />
        </Button>
        <CloseButton onClose={onClose} />
      </div>
    </Card>
  );
}

export function FruitEntryView({ entry, onEdit, onCancel, onClose }: FruitEntryViewProps) {
  const query = useFruitEntry(entry === 'bad' ? null : entry);
  if (entry === 'bad') return <NotFound error={NOT_FOUND} onClose={onClose} />;
  if (query.isPending) return <LoadingState />;
  if (query.isError) {
    const error = uiErrorOf(query.error);
    return error.errors.some((item) => item.code === 'ERR_NOT_FOUND') ? (
      <NotFound error={error} onClose={onClose} />
    ) : (
      <ErrorState
        title={<Localized id="fruit-entry-failed" />}
        error={error}
        onRetry={() => {
          void query.refetch();
        }}
      />
    );
  }
  return <Saved row={query.data} onEdit={onEdit} onCancel={onCancel} onClose={onClose} />;
}
