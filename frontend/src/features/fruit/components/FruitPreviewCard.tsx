// What the entry on the پھل form will do, before it is saved (PRD FR-16.10, FR-16.11, C27): the
// value, each partner's part with his total now and after, who is credited, and the lot's balance.
// For one partner the part comes off his total only; for the shop's guests the value is shared
// equally between the partners, not by the commission shares (C27). Every figure and every working
// is the backend's (the same function as the save); the screen shows them and works nothing out.

import { Localized, useLocalization } from '@fluent/react';
import { displayName, useLanguage } from '../../../shared/i18n';
import { CalcInfo, Card, Fact, Notice, Num, Table, type TableColumn } from '../../../shared/ui';
import { SignedNum } from '../../cashbook';
import type { FruitPreview } from '../types';

type Part = NonNullable<FruitPreview['numbers']>['parts'][number];

interface FruitPreviewCardProps {
  /** The backend's answer, or undefined while there is nothing to ask about. */
  preview: FruitPreview | undefined;
  /** The entry is for the shop's guests. */
  guests: boolean;
}

export function FruitPreviewCard({ preview, guests }: FruitPreviewCardProps) {
  const language = useLanguage();
  const { l10n } = useLocalization();
  const numbers = preview?.numbers ?? null;
  if (preview === undefined) {
    return (
      <Card compact title={<Localized id="fruit-numbers-title" />}>
        <Localized id="fruit-numbers-empty">
          <p className="type-body text-ink-muted" />
        </Localized>
      </Card>
    );
  }
  if (numbers === null) {
    return (
      <Card compact title={<Localized id="fruit-numbers-title" />}>
        <Localized id="fruit-numbers-incomplete">
          <p className="type-body text-ink-muted" />
        </Localized>
      </Card>
    );
  }
  const credited = numbers.credit.pukhta ?? numbers.credit.account;
  // Each partner's part with his total now and after (plan 39: the shared table, figures centred).
  const partColumns: TableColumn<Part>[] = [
    {
      id: 'partner',
      header: <Localized id="fruit-parts-col-partner" />,
      cell: (part) => displayName(part.partner, language),
    },
    {
      id: 'amount',
      header: <Localized id="fruit-parts-col-amount" />,
      kind: 'number',
      size: 'money',
      cell: (part) => <Num paisa={part.amount_paisa} />,
    },
    {
      id: 'now',
      header: <Localized id="fruit-parts-col-now" />,
      kind: 'number',
      size: 'money',
      cell: (part) => <SignedNum paisa={part.total_before_paisa} />,
    },
    {
      id: 'after',
      header: <Localized id="fruit-parts-col-after" />,
      kind: 'number',
      size: 'money',
      cell: (part) => <SignedNum paisa={part.total_after_paisa} />,
    },
  ];
  return (
    <Card compact title={<Localized id="fruit-numbers-title" />}>
      <dl className="grid grid-cols-2 gap-3">
        <Fact label={<Localized id="fruit-numbers-value" />}>
          <span className="flex items-center gap-1">
            <Num paisa={numbers.value_paisa} />
            <CalcInfo working={numbers.value_working} />
          </span>
        </Fact>
        <Fact
          label={
            <Localized id="fruit-numbers-credit" vars={{ name: displayName(credited, language) }} />
          }
        >
          {displayName(credited, language)}
        </Fact>
        {numbers.lot === null ? null : (
          <>
            <Fact label={<Localized id="fruit-numbers-lot-before" />}>
              <Num>{String(numbers.lot.balance_before)}</Num>
            </Fact>
            <Fact label={<Localized id="fruit-numbers-lot-after" />}>
              <Num>{String(numbers.lot.balance_after)}</Num>
            </Fact>
          </>
        )}
      </dl>
      <div className="flex flex-col gap-2">
        <div className="flex items-center gap-1">
          {guests ? (
            <Localized id="fruit-parts-title-guests">
              <h3 className="type-label text-ink-muted" />
            </Localized>
          ) : (
            <Localized
              id="fruit-parts-title-home"
              vars={{ name: displayName(numbers.parts[0]?.partner ?? credited, language) }}
            >
              <h3 className="type-label text-ink-muted" />
            </Localized>
          )}
          {numbers.parts_working === null ? null : <CalcInfo working={numbers.parts_working} />}
        </div>
        <Table
          label={
            guests
              ? l10n.getString('fruit-parts-title-guests')
              : l10n.getString('fruit-parts-title-home', {
                  name: displayName(numbers.parts[0]?.partner ?? credited, language),
                })
          }
          columns={partColumns}
          rows={numbers.parts}
          rowKey={(part) => String(part.partner.id)}
        />
        {guests ? (
          <Notice tone="info">
            <Localized id="fruit-parts-note" />
          </Notice>
        ) : null}
      </div>
    </Card>
  );
}
