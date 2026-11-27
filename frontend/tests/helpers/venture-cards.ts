// Test helper for plan 97 item 5: the garden card and the trip card of the summary, as the backend sends them in
// `BikriPreview.gardens` and `BikriPreview.trips` (PRD FR-12.52). The worked numbers are the PRD's: a garden holding 12,000
// جمع whose lots credit 56,400 holds 68,400 جمع after (E2E-40); a trip lot of 9,000 with 540 of commission credits 8,460
// (E2E-34). The shapes are those of the generated bindings.

import type { BalanceSide } from '../../src/bindings/BalanceSide';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import type { GardenCardLines } from '../../src/bindings/GardenCardLines';
import type { TripCardLines } from '../../src/bindings/TripCardLines';
import { paisaForTest } from './paisa';
import { working } from './trips-data';

const rupees = (value: number) => paisaForTest(value * 100);

export const GARDEN_ACCOUNT = 77;

export interface GardenCardOptions {
  id?: number;
  account?: number;
  previous?: { rupees: number; side: BalanceSide };
  credit?: number;
  next?: { rupees: number; side: BalanceSide };
}

export function gardenCard(options: GardenCardOptions = {}): GardenCardLines {
  const previous = options.previous ?? { rupees: 12_000, side: 'cr' };
  const next = options.next ?? { rupees: 68_400, side: 'cr' };
  const credit = options.credit ?? 56_400;
  return {
    garden: {
      id: options.id ?? 5,
      number: 1,
      code: '0080',
      name_ur: 'شاہ پور باغ',
      name_en: 'Shahpur Garden',
      account: {
        id: options.account ?? GARDEN_ACCOUNT,
        code: '0080',
        name_ur: 'شاہ پور باغ',
        name_en: 'Shahpur Garden',
      },
    },
    previous_paisa: rupees(previous.rupees),
    previous_side: previous.side,
    credit_paisa: rupees(credit),
    new_paisa: rupees(next.rupees),
    new_side: next.side,
    working: working('working-garden-new-balance', [
      { name: 'previous', value: { kind: 'money', paisa: rupees(-previous.rupees) } },
      { name: 'credit', value: { kind: 'money', paisa: rupees(credit) } },
      { name: 'result', value: { kind: 'money', paisa: rupees(-next.rupees) } },
    ]),
  };
}

export interface TripCardOptions {
  id?: number;
  code?: string;
  gross?: number;
  deductions?: number;
  credit?: number;
}

export function tripCard(options: TripCardOptions = {}): TripCardLines {
  const gross = options.gross ?? 9_000;
  const deductions = options.deductions ?? 540;
  const credit = options.credit ?? 8_460;
  return {
    trip: {
      id: options.id ?? 3,
      number: 1,
      code: options.code ?? '0070',
      name_ur: 'سفر 0070',
      name_en: 'Trip 0070',
    },
    gross_paisa: rupees(gross),
    deductions_paisa: rupees(deductions),
    credit_paisa: rupees(credit),
    working: working('working-trip-card-credit', [
      { name: 'gross', value: { kind: 'money', paisa: rupees(gross) } },
      { name: 'deductions', value: { kind: 'money', paisa: rupees(deductions) } },
      { name: 'result', value: { kind: 'money', paisa: rupees(credit) } },
    ]),
  };
}

/** A preview with the garden and trip cards put in. */
export function withCards(
  preview: BikriPreview,
  gardens: readonly GardenCardLines[],
  trips: readonly TripCardLines[],
): BikriPreview {
  return { ...preview, gardens: [...gardens], trips: [...trips] };
}
