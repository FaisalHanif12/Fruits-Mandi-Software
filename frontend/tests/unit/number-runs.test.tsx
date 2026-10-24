// A date or a range inside a text cell keeps its digit order in a right-to-left cell
// (Code Standards F25, NFR-URD-13). The text is not changed: only the run gets its own isolate.

import { render } from '@testing-library/react';
import { createElement } from 'react';
import { describe, expect, it } from 'vitest';
import { NumberRuns, hasNumberRun } from '../../src/shared/grid/NumberRuns';

const TWO_DATES = 'سفر-1 10-10-2026 تا 12-10-2026';

describe('number runs in a text cell', () => {
  it('finds a date and a range, and leaves a lone number alone', () => {
    expect(hasNumberRun('سفر-1 10-10-2026')).toBe(true);
    expect(hasNumberRun('Trip 5/6')).toBe(true);
    expect(hasNumberRun('سفر-1')).toBe(false);
    expect(hasNumberRun('Lot 12')).toBe(false);
    expect(hasNumberRun(42)).toBe(false);
    expect(hasNumberRun(null)).toBe(false);
  });

  it('puts each run in a left-to-right isolate and keeps the text the same', () => {
    const { container } = render(createElement(NumberRuns, { text: TWO_DATES }));
    const runs = Array.from(container.querySelectorAll('bdi[dir="ltr"]')).map(
      (run) => run.textContent,
    );
    expect(runs).toEqual(['10-10-2026', '12-10-2026']);
    expect(container.textContent).toBe(TWO_DATES);
  });
});
