// What the entry on the پھل form will do, before it is saved (PRD FR-16.10, FR-16.11, C27): the
// value, each partner's part with his total now and after, who is credited, and the lot's balance.
// For one partner the part comes off his total only; for the shop's guests the value is shared
// equally between the partners, not by the commission shares (C27). Every figure and every working
// is the backend's (the same function as the save); the screen shows them and works nothing out.

import { Localized } from '@fluent/react';
import { displayName, useLanguage } from '../../../shared/i18n';
import { CalcInfo, Card, Fact, Notice, Num } from '../../../shared/ui';
import { SignedNum } from '../../cashbook';
import type { FruitPreview } from '../types';

interface FruitPreviewCardProps {
  /** The backend's answer, or undefined while there is nothing to ask about. */
  preview: FruitPreview | undefined;
  /** The entry is for the shop's guests. */
  guests: boolean;
}

export function FruitPreviewCard({ preview, guests }: FruitPreviewCardProps) {
  const language = useLanguage();
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
        <table className="w-full type-body">
          <thead>
            <tr className="text-ink-muted">
              <Localized id="fruit-parts-col-partner">
                <th scope="col" className="py-1 pe-2 text-start font-normal" />
              </Localized>
              <Localized id="fruit-parts-col-amount">
                <th scope="col" className="py-1 text-end font-normal" />
              </Localized>
              <Localized id="fruit-parts-col-now">
                <th scope="col" className="py-1 text-end font-normal" />
              </Localized>
              <Localized id="fruit-parts-col-after">
                <th scope="col" className="py-1 text-end font-normal" />
              </Localized>
            </tr>
          </thead>
          <tbody>
            {numbers.parts.map((part) => (
              <tr key={part.partner.id} className="border-t border-border">
                <th scope="row" className="py-1 pe-2 text-start font-normal">
                  {displayName(part.partner, language)}
                </th>
                <td className="py-1 text-end">
                  <Num paisa={part.amount_paisa} />
                </td>
                <td className="py-1 text-end">
                  <SignedNum paisa={part.total_before_paisa} />
                </td>
                <td className="py-1 text-end">
                  <SignedNum paisa={part.total_after_paisa} />
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {guests ? (
          <Notice tone="info">
            <Localized id="fruit-parts-note" />
          </Notice>
        ) : null}
      </div>
    </Card>
  );
}
