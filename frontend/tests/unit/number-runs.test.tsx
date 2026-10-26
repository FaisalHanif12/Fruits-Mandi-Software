// Every number inside a text cell keeps its digit order and its shape in a right-to-left cell (Code
// Standards F25, NFR-URD-13): a date, a range, and also a lone number such as the 1 of "سفر-1"
// (Umar, 26-10-2026, see NumberRuns.tsx). The text is not changed: only each run gets its own
// isolate. This file once said a lone number is left alone; that was the rule before 26-10-2026.

import { render } from '@testing-library/react';
import { createElement } from 'react';
import { describe, expect, it } from 'vitest';
import { NumberRuns, hasNumberRun } from '../../src/shared/grid/NumberRuns';

const TWO_DATES = 'سفر-1 10-10-2026 تا 12-10-2026';

describe('number runs in a text cell', () => {
  it('finds a date, a range and a lone number; only words, or something that is not text, hold none', () => {
    expect(hasNumberRun('سفر-1 10-10-2026')).toBe(true);
    expect(hasNumberRun('Trip 5/6')).toBe(true);
    expect(hasNumberRun('سفر-1')).toBe(true);
    expect(hasNumberRun('Lot 12')).toBe(true);
    expect(hasNumberRun('کیلا')).toBe(false);
    expect(hasNumberRun(42)).toBe(false);
    expect(hasNumberRun(null)).toBe(false);
  });

  it('puts each run, the 1 of the trip and both dates, in a left-to-right isolate and keeps the text the same', () => {
    const { container } = render(createElement(NumberRuns, { text: TWO_DATES }));
    const runs = Array.from(container.querySelectorAll('bdi[dir="ltr"]')).map(
      (run) => run.textContent,
    );
    expect(runs).toEqual(['1', '10-10-2026', '12-10-2026']);
    expect(container.textContent).toBe(TWO_DATES);
  });
});
