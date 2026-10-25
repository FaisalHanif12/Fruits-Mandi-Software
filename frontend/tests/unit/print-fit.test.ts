// A printed table wider than its sheet is zoomed down to fit (round 14, PRD FR-19.3, NFR-USE-15).

import { describe, expect, it } from 'vitest';
import { MIN_ZOOM, fitZoom } from '../../src/features/reports/print/fit';

describe('fitZoom', () => {
  it('leaves a table that fits alone', () => {
    expect(fitZoom(700, 700)).toBeNull();
    expect(fitZoom(500, 700)).toBeNull();
  });

  it('brings a wide table inside its sheet: 1000 wide where 700 is left is 0.7', () => {
    expect(fitZoom(1000, 700)).toBe(0.7);
  });

  it('rounds down to two decimals, so the table is never a hair too wide', () => {
    // 700 / 758 = 0.9234...
    expect(fitZoom(758, 700)).toBe(0.92);
  });

  it('never goes below the smallest readable zoom', () => {
    expect(fitZoom(10_000, 100)).toBe(MIN_ZOOM);
  });

  it('does nothing with sizes that are not real (a page with no layout)', () => {
    expect(fitZoom(0, 700)).toBeNull();
    expect(fitZoom(700, 0)).toBeNull();
    expect(fitZoom(Number.NaN, 700)).toBeNull();
    expect(fitZoom(700, Number.NaN)).toBeNull();
  });
});
