// The card of a garden and the card of a trip in the summary (PRD FR-12.52, plan 97 item 5, Umar 24-11-2026), in the same
// grid as the seller cards (`AdvanceTicks`).
//
// - A garden card: its name, پچھلا بقایا (its account before this بکری, with its side), آج کی صافی بکری (the credit of its
//   lots in this بکری) and نیا بقایا. A garden lot takes no نقد, so there is no "paid on the spot" row.
// - A trip card: its number, خام بکری, کٹوتی and آج کی صافی بکری, and the line that says where that credit goes: to the trip's
//   partners by their share. No partner figure is on the card (FR-17.11).
//
// Every figure is the backend's, worked out by the function that posts the بکری (Code Standards B5, F42): the row of a
// figure that has a working shows its ⓘ. The frontend adds nothing (CLAUDE.md rule 1).

import { Localized } from '@fluent/react';
import { useId } from 'react';
import type { GardenCardLines } from '../../../bindings/GardenCardLines';
import type { TripCardLines } from '../../../bindings/TripCardLines';
import { displayName, useLanguage } from '../../../shared/i18n';
import { Balance, Code, Num } from '../../../shared/ui';
import { CardLine } from './CardLine';

const CARD_CLASS =
  'flex min-w-0 flex-col gap-2 rounded-2 border border-border bg-surface-sunken p-3';

interface GardenCardProps {
  card: GardenCardLines;
}

export function GardenCard({ card }: GardenCardProps) {
  const language = useLanguage();
  const name = displayName(card.garden, language);
  return (
    <section aria-label={name} data-garden-card={card.garden.id} className={CARD_CLASS}>
      <h3 className="type-label text-ink">{name}</h3>
      <dl className="flex flex-col" aria-label={name}>
        <CardLine labelId="bikri-seller-paid-previous">
          <Balance balance={{ amount_paisa: card.previous_paisa, side: card.previous_side }} />
        </CardLine>
        <CardLine labelId="bikri-seller-paid-net">
          <Num paisa={card.credit_paisa} />
        </CardLine>
        <CardLine labelId="bikri-seller-paid-new" working={card.working} emphasis>
          <Balance balance={{ amount_paisa: card.new_paisa, side: card.new_side }} />
        </CardLine>
      </dl>
    </section>
  );
}

interface TripCardProps {
  card: TripCardLines;
}

export function TripCard({ card }: TripCardProps) {
  const titleId = useId();
  return (
    <section aria-labelledby={titleId} data-trip-card={card.trip.id} className={CARD_CLASS}>
      <Localized
        id="bikri-trip-card-title"
        vars={{ code: card.trip.code }}
        elems={{ code: <Code /> }}
      >
        <h3 id={titleId} className="type-label text-ink" />
      </Localized>
      <dl className="flex flex-col">
        <CardLine labelId="glossary-kham-bikri">
          <Num paisa={card.gross_paisa} />
        </CardLine>
        <CardLine labelId="bikri-trip-card-deductions">
          <Num paisa={card.deductions_paisa} />
        </CardLine>
        <CardLine labelId="bikri-seller-paid-net" working={card.working} emphasis>
          <Num paisa={card.credit_paisa} />
        </CardLine>
      </dl>
      <Localized id="bikri-trip-card-partners">
        <p className="type-body text-ink-muted" />
      </Localized>
    </section>
  );
}
