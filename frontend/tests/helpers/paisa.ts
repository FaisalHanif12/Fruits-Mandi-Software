// Test-only: build a Paisa from a number, the way a JSON reply from the backend arrives.
// App code never builds Paisa; it only receives it from the backend (Code Standards F3, M12).

import { chartValueOfPaisa, type Paisa } from '../../src/shared/format/money';

/** Example: `paisaForTest(1_200_000)` is 12,000 rupees. */
export function paisaForTest(amount: number): Paisa {
  return amount as unknown as Paisa;
}

/**
 * Test-only: the plain number inside a Paisa, for a test double that works out a total the way the
 * backend would (the statement helpers). It uses the one accessor money.ts gives for drawing a chart,
 * so no cast touches a Paisa. App code never does this (Code Standards F3, M12).
 */
export function plainPaisaForTest(paisa: Paisa): number {
  return chartValueOfPaisa(paisa);
}
