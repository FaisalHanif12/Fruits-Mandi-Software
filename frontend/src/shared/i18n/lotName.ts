// The name of a lot as words (PRD FR-12.45, plan 71 L1 to L4, L8): the seller, the item and the
// counter, for example "Ehsan, Mango-2" or "احسان، آم-2". The counter belongs to one seller and one
// item for the seller's whole life, and the backend gives it; this only draws it. The seller and the
// item come through `displayName` (with the tag, Code Standards F38). The words that join them (the
// comma, the dash) are in Fluent (`bikri-lot-name`, `bikri-lot-who`, `bikri-lot-item`), so Urdu and
// English each use their own. Every screen that names a lot (the sale book, the goods left, the
// later sale, edits, lists, trips, gardens, fruit) uses this one place.

import type { ReactLocalization } from '@fluent/react';
import { useLocalization } from '@fluent/react';
import { useCallback } from 'react';
import type { Language } from '../../i18n';
import { displayName, type Named } from './displayName';
import { useLanguage } from './language';

/**
 * Fluent wraps every variable in invisible isolation marks. A name made of two of them, put into a
 * label that wraps it again, would be an isolate whose own letters are all inside other isolates, so
 * the right-to-left reader would take it for left-to-right and show the item before the seller.
 * The marks of the inner message are taken off, so the label's own isolation reads the name's first
 * letter and sets its direction (the seller's script).
 */
export function withoutIsolation(text: string): string {
  return text.replace(/[⁦-⁩]/g, '');
}

/** A seller or an item: a record with both names (and maybe a tag), or just the two names. */
export type LotPart = Named;

/** A Fluent message as words: the shape of `l10n.getString` and of the `text` of a list column. */
export type Say = (id: string, vars?: Record<string, string>) => string;

/**
 * "Ehsan, Mango-2" when both are chosen and the lot has a counter; "Ehsan, Mango" when it has none
 * yet; the one name that is chosen; or the empty text when neither is.
 */
export function lotNameWith(
  say: Say,
  language: Language,
  seller: LotPart | null | undefined,
  item: LotPart | null | undefined,
  counter: number | null | undefined,
): string {
  const sellerText = seller === null || seller === undefined ? null : displayName(seller, language);
  const itemText = item === null || item === undefined ? null : displayName(item, language);
  if (sellerText !== null && itemText !== null) {
    return withoutIsolation(
      counter === null || counter === undefined
        ? say('bikri-lot-who', { seller: sellerText, item: itemText })
        : say('bikri-lot-name', { seller: sellerText, item: itemText, counter: String(counter) }),
    );
  }
  return sellerText ?? itemText ?? '';
}

/** The item with its counter, "Mango-2", for a list that shows the seller in a column of its own. */
export function lotItemWith(
  say: Say,
  language: Language,
  item: LotPart,
  counter: number | null | undefined,
): string {
  const name = displayName(item, language);
  return counter === null || counter === undefined
    ? name
    : withoutIsolation(say('bikri-lot-item', { item: name, counter: String(counter) }));
}

/** `lotNameWith` for a component that has `l10n`. */
export function lotNameText(
  l10n: ReactLocalization,
  language: Language,
  seller: LotPart | null | undefined,
  item: LotPart | null | undefined,
  counter: number | null | undefined,
): string {
  return lotNameWith((id, vars) => l10n.getString(id, vars), language, seller, item, counter);
}

/** `lotItemWith` for a component that has `l10n`. */
export function lotItemText(
  l10n: ReactLocalization,
  language: Language,
  item: LotPart,
  counter: number | null | undefined,
): string {
  return lotItemWith((id, vars) => l10n.getString(id, vars), language, item, counter);
}

/** `lotNameText` and `lotItemText` for a component, in the language of the app. */
export function useLotNames(): {
  lotName: (
    seller: LotPart | null | undefined,
    item: LotPart | null | undefined,
    counter: number | null | undefined,
  ) => string;
  lotItem: (item: LotPart, counter: number | null | undefined) => string;
} {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const lotName = useCallback(
    (
      seller: LotPart | null | undefined,
      item: LotPart | null | undefined,
      counter: number | null | undefined,
    ) => lotNameText(l10n, language, seller, item, counter),
    [l10n, language],
  );
  const lotItem = useCallback(
    (item: LotPart, counter: number | null | undefined) =>
      lotItemText(l10n, language, item, counter),
    [l10n, language],
  );
  return { lotName, lotItem };
}
