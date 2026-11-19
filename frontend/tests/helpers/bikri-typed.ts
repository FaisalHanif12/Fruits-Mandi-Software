// A test double of `bikri_preview` for a commission or لاگا typed on a sale line (PRD FR-11.10): it answers
// like the real backend (services test `a_percent_typed_on_the_line_is_manual_and_the_split_follows`) for
// the first line of Event 1 (20 at 600, base 12,000). A percent typed on the seller commission is method
// manual, source typed, the rate in basis points and the amount worked out from the base. The numbers are
// plain numbers in a test double; the screen works out nothing. No test library is imported, so the
// development page can use it too.

import type { BikriDraft } from '../../src/bindings/BikriDraft';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import { previewFor } from './bikri-data';
import { paisaForTest } from './paisa';

/** The base of the first line of Event 1 in paisa: 20 at 600. */
const BASE_PAISA = 1_200_000;

/** The quantity of the first line of Event 1. */
const QTY = 20;
/** What the rules give the first line of Event 1: buyer commission 6% (720 rupees) and لاگا 8 for each نگ (160). */
const RULE_BUYER_COMM_PAISA = 72_000;
const RULE_LAGA_PAISA = 16_000;

/** A percent typed as text, in basis points ("4" is 400); nothing when it is empty or not a number. */
function bpsOf(text: string): number | null {
  const percent = Number(text);
  return text.trim() === '' || !Number.isFinite(percent) ? null : Math.round(percent * 100);
}

/**
 * The answer for a draft as the real backend gives it when the first line has a seller commission, a buyer
 * commission or a لاگا typed on it (PRD FR-11.10): the typed charge is method manual, source typed, and the
 * buyer total follows the buyer commission and the لاگا.
 */
export function answerWithTyped(draft: BikriDraft): BikriPreview {
  const preview = previewFor(draft);
  const sale = draft.lots[0]?.sales[0];
  const calc = preview.lots[0]?.lines[0]?.calc;
  if (sale === undefined || calc == null) return preview;
  const typedSeller = sale.seller_comm;
  const sellerBps = typedSeller?.method === 'percent' ? bpsOf(typedSeller.value) : null;
  if (sellerBps !== null) {
    calc.seller_comm = {
      ...calc.seller_comm,
      method: 'manual',
      value_bps: sellerBps,
      value_paisa: null,
      amount: paisaForTest(Math.round((BASE_PAISA * sellerBps) / 10_000)),
      source: 'typed',
      rule_id: null,
      rule_label: null,
    };
  }
  let buyerComm = RULE_BUYER_COMM_PAISA;
  const typedBuyer = sale.buyer_comm;
  const buyerBps = typedBuyer?.method === 'percent' ? bpsOf(typedBuyer.value) : null;
  if (buyerBps !== null) {
    buyerComm = Math.round((BASE_PAISA * buyerBps) / 10_000);
    calc.buyer_comm = {
      ...calc.buyer_comm,
      method: 'manual',
      value_bps: buyerBps,
      value_paisa: null,
      amount: paisaForTest(buyerComm),
      source: 'typed',
      rule_id: null,
      rule_label: null,
    };
  }
  let laga = RULE_LAGA_PAISA;
  const typedLaga = sale.laga;
  const lagaRupees =
    typedLaga?.method === 'per_unit' ? Number(typedLaga.value.replaceAll(',', '')) : NaN;
  if (
    typedLaga?.method === 'per_unit' &&
    typedLaga.value.trim() !== '' &&
    Number.isFinite(lagaRupees)
  ) {
    laga = QTY * lagaRupees * 100;
    calc.laga = {
      ...calc.laga,
      method: 'manual',
      value_bps: null,
      value_paisa: paisaForTest(lagaRupees * 100),
      amount: paisaForTest(laga),
      source: 'typed',
      rule_id: null,
      rule_label: null,
    };
  }
  // The buyer total is the value, the buyer commission and the لاگا; nothing of it is paid on the spot.
  const total = BASE_PAISA + buyerComm + laga;
  calc.buyer_total = paisaForTest(total);
  calc.left_paisa = paisaForTest(total);
  return preview;
}
