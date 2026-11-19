// How long a question took, as the question log shows it (plan 51 B2). Whole-number maths only.

import { describe, expect, it } from 'vitest';
import { shownDuration } from '../../src/features/assistant/timing';

describe('shownDuration', () => {
  it.each([
    [0, '0', 'ms'],
    [4, '4', 'ms'],
    [999, '999', 'ms'],
    [1000, '1.0', 's'],
    [1049, '1.0', 's'],
    [1050, '1.1', 's'],
    [8350, '8.4', 's'],
    [8349, '8.3', 's'],
    [18309, '18.3', 's'],
    [60000, '60.0', 's'],
    [90000, '90.0', 's'],
  ])('%i ms is %s %s', (millis, value, unit) => {
    expect(shownDuration(millis)).toEqual({ value, unit });
  });
});
