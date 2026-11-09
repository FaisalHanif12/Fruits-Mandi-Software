// The name of a lot as words, for every label that points at a lot (PRD FR-12.6): the seller and
// the item, for example "Ehsan, Apple" or "احسان، سیب". One seller can own two lots on a truck, so the
// number alone does not say which lot a label means. The seller comes through `displayName` (with
// his tag, Code Standards F38). The words that join the two (the comma) are in Fluent
// (`bikri-lot-who`), so Urdu and English each use their own.

import { useLocalization } from '@fluent/react';
import { useCallback } from 'react';
import type { SavedLot } from '../../bindings/SavedLot';
import type { EntryRow } from '../../shared/grid';
import { displayName, useLanguage } from '../../shared/i18n';
import type { Language } from '../../i18n';
import { LOT } from './types';

/**
 * Fluent wraps every variable in invisible isolation marks. A name made of two of them, put into a
 * label that wraps it again, would be an isolate whose own letters are all inside other isolates, so
 * the right-to-left reader would take it for left-to-right and show the item before the seller.
 * The marks of the inner message are taken off, so the label's own isolation reads the name's first
 * letter and sets its direction (the seller's script).
 */
function withoutIsolation(text: string): string {
  return text.replace(/[\u2066-\u2069]/g, '');
}

/** What the Fluent messages of a lot label take: its number, its name and whether it has one. */
export interface LotLabelVars {
  lot: string;
  who: string;
  /** `yes` when the lot has a name to show; the message then adds it after the number. */
  named: 'yes' | 'no';
}

/** The seller of a lot as a name: his account's name, or null while none is chosen. */
function sellerName(row: EntryRow, language: Language): string | null {
  const seller = row.picked?.[LOT.seller];
  return seller === undefined ? null : displayName(seller, language);
}

/** The item of a lot as a name, or null while none is chosen. */
function itemName(row: EntryRow, language: Language): string | null {
  const item = row.picked?.[LOT.item];
  return item === undefined ? null : displayName(item, language);
}

/**
 * The words that name a lot: "Ehsan, Apple". It is the empty text while neither the seller nor the
 * item is chosen, and then a label shows the number alone.
 */
export function useLotWho(): (row: EntryRow | undefined) => string {
  const { l10n } = useLocalization();
  const language = useLanguage();
  return useCallback(
    (row) => {
      if (row === undefined) return '';
      const seller = sellerName(row, language);
      const item = itemName(row, language);
      // Both chosen: the two names joined (the comma is Fluent's). One: that name alone. None: no name.
      if (seller !== null && item !== null) {
        return withoutIsolation(l10n.getString('bikri-lot-who', { seller, item }));
      }
      return seller ?? item ?? '';
    },
    [l10n, language],
  );
}

/** The values of a lot label: the lot's number and its name (empty text: the number stands alone). */
export function lotLabelVars(number: number, who: string): LotLabelVars {
  return { lot: String(number), who, named: who === '' ? 'no' : 'yes' };
}

/**
 * The same words for a saved lot: its seller and its item. It is the empty text only when the lot
 * has neither, which a saved lot never has.
 */
export function useSavedLotWho(): (lot: SavedLot) => string {
  const { l10n } = useLocalization();
  const language = useLanguage();
  return useCallback(
    (lot) =>
      withoutIsolation(
        l10n.getString('bikri-lot-who', {
          seller: displayName(lot.seller, language),
          item: displayName(lot.item, language),
        }),
      ),
    [l10n, language],
  );
}
