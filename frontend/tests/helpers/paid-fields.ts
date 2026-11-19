// Test double: the paid fields of a sale line (PRD FR-12.44, plan 65) on a preview that was written
// before the backend had them. A line that has `paid_paisa` already, such as every line of the
// backend's own files written for plan 65, is left exactly as it is. A line without them gets what
// the old toggle meant: a cash line is paid in full, a credit line is paid 0 (PRD FR-12.44, "old
// saved lines keep their meaning"). Nothing is worked out: the figures are copied from the line.
// When every fixture is the backend's new file, this does nothing.

import type { BikriPreview } from '../../src/bindings/BikriPreview';
import type { Working } from '../../src/bindings/Working';
import { paisaForTest } from './paisa';

/** A working with one money argument, in the shape the backend sends. */
function working(messageId: string, result: unknown): Working {
  return {
    message_id: messageId,
    args: [{ name: 'result', value: { kind: 'money', paisa: result as never } }],
    rule_id: null,
  };
}

/** The preview with the paid fields on every finished line that lacks them. */
export function ensurePaid(preview: BikriPreview): BikriPreview {
  const copy = structuredClone(preview);
  for (const lot of copy.lots) {
    for (const line of lot.lines) {
      const calc = line.calc as unknown as Record<string, unknown> | null;
      if (calc === null || calc['paid_paisa'] !== undefined) continue;
      const cash = calc['payment_type'] === 'cash';
      const total = calc['buyer_total'];
      const zero = paisaForTest(0);
      calc['paid_paisa'] = cash ? total : zero;
      calc['left_paisa'] = cash ? zero : total;
      calc['paid_fixed'] = false;
      const workings = calc['workings'] as Record<string, unknown>;
      workings['paid'] = working('working-paid-typed', calc['paid_paisa']);
      workings['left'] = working('working-left', calc['left_paisa']);
    }
  }
  return copy;
}
