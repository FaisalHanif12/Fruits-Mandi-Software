// Test data: the answer of `partners_open_ventures` (plan 99, PRD FR-17.3, FR-19.11) made from a list of
// cards, as a healthy backend answers: the cards made in the days asked for, newest first, one page, and
// the totals over all of them, the gardens' balance and the trips' result kept apart. The sums are this
// test double's (a test double works on rupees); the screen only shows what comes back (CLAUDE.md rule 1).
// No test runner in it, so the development pages can use it too.

import type { PartnerOpenVentures } from '../../src/bindings/PartnerOpenVentures';
import type { PartnerVentureCard } from '../../src/bindings/PartnerVentureCard';
import type { PartnersOpenVenturesInput } from '../../src/bindings/PartnersOpenVenturesInput';
import { paisaForTest, plainPaisaForTest } from './paisa';

/** `DD-MM-YYYY` as `YYYY-MM-DD`, so two days compare as text. */
const isoOf = (day: string) => day.split('-').reverse().join('-');

export function openVenturesAnswer(
  cards: readonly PartnerVentureCard[],
  input: PartnersOpenVenturesInput,
): PartnerOpenVentures {
  const found = cards
    .filter(
      (card) =>
        (input.from === null || card.entry_date >= isoOf(input.from)) &&
        (input.to === null || card.entry_date <= isoOf(input.to)),
    )
    .sort((a, b) => b.entry_date.localeCompare(a.entry_date) || b.venture_id - a.venture_id);
  const limit = input.limit === 0 ? 100 : input.limit;
  const sum = (pick: (card: PartnerVentureCard) => number) =>
    paisaForTest(found.reduce((total, card) => total + pick(card), 0));
  return {
    rows: found.slice(input.offset, input.offset + limit),
    total_count: found.length,
    totals: {
      paid_shop_paisa: sum((card) => plainPaisaForTest(card.paid_shop_paisa)),
      paid_pocket_paisa: sum((card) => plainPaisaForTest(card.paid_pocket_paisa)),
      money_back_paisa: sum((card) => plainPaisaForTest(card.money_back_paisa)),
      gardens_balance_paisa: sum((card) =>
        card.kind === 'garden' ? plainPaisaForTest(card.balance_paisa) : 0,
      ),
      trips_result_paisa: sum((card) =>
        card.kind === 'trip' ? plainPaisaForTest(card.balance_paisa) : 0,
      ),
    },
  };
}
