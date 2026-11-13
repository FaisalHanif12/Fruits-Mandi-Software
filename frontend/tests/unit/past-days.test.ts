// Which days of the window of پچھلی بکری are open (plan 75 7.7, PRD FR-12.46 v1.39): the default, what the
// Owner opens or closes by hand, and the one day a go-to-date or a way back from an edit asks for. Pure; no
// money (the pills of a day are the backend's figures).

import { describe, expect, it } from 'vitest';
import type { BikriDay } from '../../src/bindings/BikriDay';
import {
  NO_OVERRIDES,
  isDayOpen,
  toggleDay,
  withDayOpen,
} from '../../src/features/bikri/past-days';
import { paisaForTest } from '../helpers/paisa';

const day = (date: string, count: number): BikriDay => ({
  date,
  count,
  gross_paisa: paisaForTest(count * 100),
});
const SALES = day('2026-10-09', 2);
const OTHER = day('2026-10-07', 1);
const EMPTY = day('2026-10-08', 0);

describe('isDayOpen', () => {
  it('a day with بکری is closed by default and open while the filters or the tick open the days', () => {
    expect(isDayOpen(SALES, false, NO_OVERRIDES)).toBe(false);
    expect(isDayOpen(SALES, true, NO_OVERRIDES)).toBe(true);
  });

  it('a day with no بکری is never open, whatever is asked', () => {
    expect(isDayOpen(EMPTY, false, NO_OVERRIDES)).toBe(false);
    expect(isDayOpen(EMPTY, true, NO_OVERRIDES)).toBe(false);
    expect(isDayOpen(EMPTY, false, withDayOpen(EMPTY.date, NO_OVERRIDES))).toBe(false);
  });

  it('what the Owner did by hand beats the default', () => {
    const closedByHand = toggleDay(SALES, true, NO_OVERRIDES);
    expect(isDayOpen(SALES, true, closedByHand)).toBe(false);
    const openedByHand = toggleDay(SALES, false, NO_OVERRIDES);
    expect(isDayOpen(SALES, false, openedByHand)).toBe(true);
  });
});

describe('toggleDay', () => {
  it('opens a closed day and closes an open one, and leaves the other days as they were', () => {
    const first = toggleDay(SALES, false, NO_OVERRIDES);
    expect(isDayOpen(SALES, false, first)).toBe(true);
    expect(isDayOpen(OTHER, false, first)).toBe(false);
    const second = toggleDay(SALES, false, first);
    expect(isDayOpen(SALES, false, second)).toBe(false);
  });

  it('several days can be open together', () => {
    const both = toggleDay(OTHER, false, toggleDay(SALES, false, NO_OVERRIDES));
    expect(isDayOpen(SALES, false, both)).toBe(true);
    expect(isDayOpen(OTHER, false, both)).toBe(true);
  });

  it('does not change what it was given', () => {
    toggleDay(SALES, false, NO_OVERRIDES);
    expect(NO_OVERRIDES.opened.size).toBe(0);
    expect(NO_OVERRIDES.closed.size).toBe(0);
  });
});

describe('withDayOpen', () => {
  it('opens the day, also one the Owner had closed by hand', () => {
    const closed = toggleDay(SALES, true, NO_OVERRIDES);
    expect(isDayOpen(SALES, true, closed)).toBe(false);
    expect(isDayOpen(SALES, true, withDayOpen(SALES.date, closed))).toBe(true);
    expect(isDayOpen(SALES, false, withDayOpen(SALES.date, NO_OVERRIDES))).toBe(true);
  });

  it('opens that day only', () => {
    const next = withDayOpen(SALES.date, NO_OVERRIDES);
    expect(isDayOpen(OTHER, false, next)).toBe(false);
  });
});
