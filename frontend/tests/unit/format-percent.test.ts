// A percent on screen (plan 09): basis points written as a person writes a percent, by digits as
// text, never by division (Code Standards M3).

import { describe, expect, it } from 'vitest';
import { formatBps } from '../../src/shared/format';

describe('formatBps', () => {
  it('writes whole percents without decimals', () => {
    expect(formatBps(600)).toBe('6');
    expect(formatBps(10000)).toBe('100');
    expect(formatBps(0)).toBe('0');
  });

  it('writes up to two decimals and drops a trailing zero', () => {
    expect(formatBps(250)).toBe('2.5');
    expect(formatBps(275)).toBe('2.75');
    expect(formatBps(5)).toBe('0.05');
    expect(formatBps(50)).toBe('0.5');
    expect(formatBps(1)).toBe('0.01');
  });
});
