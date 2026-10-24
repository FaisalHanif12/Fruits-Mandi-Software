// Closing a garden (PRD FR-12.38, FR-7.8, E64): when the goods stop coming, the Owner closes it.
// The card asks for the closing day (it starts as today) and shows, live, what the closing will do:
// the money in, the total cost, the result (a loss is red and also written), the garden account's
// balance that goes to the partners by their % (C26), and each partner's part and money back. All
// of it is the backend's preview, the same function as the save (Code Standards B5).
//
// The button opens a confirm box that says exactly what will happen; Back has the focus, so a
// stray Enter never closes a garden (FR-7.8). A garden with nothing in its account still closes,
// and nothing is posted (E64); the card says so. After the closing the garden takes no more
// entries, and the closing can be undone while its day is open.
//
// Four states: the preview loading, an error with a retry, the numbers, and a refusal (for
// example a day before the garden's last entry) shown at the day field.

import { Localized, useLocalization } from '@fluent/react';
import { TriangleAlert } from 'lucide-react';
import { useMemo, useState } from 'react';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import {
  formatPaisa,
  isNegativePaisa,
  isoToDisplayDate,
  useGrouping,
} from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { formNavigationProps } from '../../../shared/keyboard';
import {
  Button,
  Card,
  ConfirmDialog,
  DateInput,
  ErrorNotice,
  ErrorState,
  Field,
  Notice,
  Num,
  ShareAmounts,
  fieldError,
  showToast,
} from '../../../shared/ui';
import { useCloseGarden, useGardenClosePreview } from '../hooks';
import { gardenNumberText } from '../list-columns';
import type { GardenCloseInput, GardenRow } from '../types';
import { GardenPartnersCard } from './GardenPartnersCard';

interface CloseGardenCardProps {
  garden: GardenRow;
  /** Today, `YYYY-MM-DD`. */
  today: string;
}

function asUiError(items: readonly UiError['errors'][number][]): UiError | null {
  return items.length === 0
    ? null
    : {
        ok: false,
        errors: items.map((item) => ({ code: item.code, field: item.field, values: item.values })),
        ref: '',
      };
}

export function CloseGardenCard({ garden, today }: CloseGardenCardProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const grouping = useGrouping();
  const close = useCloseGarden();
  const [date, setDate] = useState(() => isoToDisplayDate(today));
  const [asking, setAsking] = useState(false);
  const [refusal, setRefusal] = useState<{ error: UiError; date: string } | null>(null);
  const say = (id: string, vars?: Record<string, string>) => l10n.getString(id, vars);

  const input: GardenCloseInput | null = useMemo(
    () =>
      date.trim() === '' ? null : { id: garden.id, version: garden.version, date: date.trim() },
    [date, garden.id, garden.version],
  );
  const view = useGardenClosePreview(input);
  const preview = input === null ? undefined : view.query.data;
  const numbers = preview?.numbers ?? null;
  const failed = input !== null && view.query.isError;
  // A refusal belongs to the day it was for; typing another day clears it.
  const shownRefusal = refusal !== null && refusal.date === date ? refusal.error : null;
  const previewError = useMemo(() => asUiError(preview?.errors ?? []), [preview?.errors]);
  const error = shownRefusal ?? previewError;
  const canClose = view.current && numbers !== null && previewError === null;
  const loss = numbers !== null && isNegativePaisa(numbers.result_paisa);

  const confirm = () => {
    if (input === null) return;
    setRefusal(null);
    close
      .mutateAsync(input)
      .then((closed) => {
        showToast(
          'success',
          l10n.getString('gardens-closed-toast', {
            number: gardenNumberText(closed.row.number, say),
          }),
        );
      })
      .catch((failure: unknown) => {
        setRefusal({ error: uiErrorOf(failure), date });
      });
  };

  return (
    <Card compact title={<Localized id="gardens-close-title" />}>
      <Localized id="gardens-close-help">
        <p className="type-body text-ink-muted" />
      </Localized>
      {error === null ? null : <ErrorNotice error={error} fieldsShown={['date']} />}
      <div {...formNavigationProps} className="grid grid-cols-2 items-end gap-4 normal:grid-cols-4">
        <Field
          label={<Localized id="gardens-close-date" />}
          hint={
            preview?.earliest_date == null ? undefined : (
              <Localized
                id="gardens-close-earliest"
                vars={{ date: isoToDisplayDate(preview.earliest_date) }}
              >
                <span />
              </Localized>
            )
          }
          error={fieldError(error, 'date')}
        >
          {(props) => <DateInput {...props} value={date} onChange={setDate} />}
        </Field>
      </div>
      {numbers === null ? (
        <Localized id="gardens-close-numbers-empty">
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : (
        <>
          <dl className="flex flex-col">
            <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-body text-ink">
              <Localized id="gardens-close-money-in">
                <dt />
              </Localized>
              <dd>
                <Num paisa={numbers.money_in_paisa} />
              </dd>
            </div>
            <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-body text-ink">
              <Localized id="gardens-close-total-cost">
                <dt />
              </Localized>
              <dd>
                <Num paisa={numbers.total_cost_paisa} />
              </dd>
            </div>
            <div
              className={`flex items-center justify-between gap-3 border-b border-border py-1 type-label ${loss ? 'text-due' : 'text-ink'}`}
            >
              <Localized id={loss ? 'glossary-tijarati-nuqsan' : 'glossary-tijarati-munafa'}>
                <dt />
              </Localized>
              <dd>
                <Num paisa={numbers.result_paisa} />
              </dd>
            </div>
            <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-body text-ink">
              <Localized id="gardens-close-balance">
                <dt />
              </Localized>
              <dd>
                <Num paisa={numbers.balance_paisa} />
              </dd>
            </div>
          </dl>
          {loss ? (
            <p className="flex items-center gap-2 type-body text-due">
              <TriangleAlert aria-hidden="true" className="size-icon-sm shrink-0" />
              <Localized id="gardens-close-loss-note">
                <span />
              </Localized>
            </p>
          ) : null}
          {numbers.posts_entry ? (
            <>
              <Localized id="gardens-close-credits-title">
                <h3 className="type-label text-ink-muted" />
              </Localized>
              <ShareAmounts
                credits={numbers.credits}
                label={l10n.getString('gardens-close-credits-title')}
              />
            </>
          ) : (
            <Notice tone="info">
              <Localized id="gardens-close-nothing-posted" />
            </Notice>
          )}
          <GardenPartnersCard results={numbers.partner_results} closed />
        </>
      )}
      {failed ? (
        <ErrorState
          title={<Localized id="gardens-preview-failed" />}
          error={uiErrorOf(view.query.error)}
          onRetry={() => {
            void view.query.refetch();
          }}
        />
      ) : null}
      <div className="flex justify-end">
        <Button
          variant="danger"
          busy={close.isPending}
          disabled={!canClose}
          onClick={() => {
            setAsking(true);
          }}
        >
          <Localized id="gardens-close-button" />
        </Button>
      </div>
      <ConfirmDialog
        open={asking}
        onOpenChange={setAsking}
        tone="danger"
        title={<Localized id="gardens-close-confirm-title" />}
        description={
          numbers === null ? null : (
            <Localized
              id="gardens-close-confirm-text"
              vars={{
                name: displayName(garden, language),
                date: date.trim(),
                balance: formatPaisa(numbers.balance_paisa, { grouping }),
              }}
            >
              <span />
            </Localized>
          )
        }
        confirmLabel={<Localized id="gardens-close-confirm" />}
        onConfirm={confirm}
      />
    </Card>
  );
}
