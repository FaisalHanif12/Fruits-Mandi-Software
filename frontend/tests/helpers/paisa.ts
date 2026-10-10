// Test-only: build a Paisa from a number, the way a JSON reply from the backend arrives.
// App code never builds Paisa; it only receives it from the backend (Code Standards F3, M12).

import type { Paisa } from '../../src/shared/format/money';

/** Example: `paisaForTest(1_200_000)` is 12,000 rupees. */
export function paisaForTest(amount: number): Paisa {
  return amount as unknown as Paisa;
}
