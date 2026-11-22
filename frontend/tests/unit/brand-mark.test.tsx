// The mark of the app (plan 75, simplified by Umar on 21-11-2026): the navy tile with only the gold mandi crate.
// The fruit (an orange, an apple and a leaf) is gone from the side bar and the sign-in panel; the fruit picture stays
// in the print headers only. The mark is the same in both languages and both sizes, and a screen reader skips it.

import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { BrandMark } from '../../src/shared/ui/BrandMark';

describe.each(['sm', 'lg'] as const)('the brand mark, size %s', (size) => {
  it('is the navy tile and the gold crate: no circle (orange, apple) and no path (leaf), no fruit colour', () => {
    const { container } = render(<BrandMark size={size} />);
    const svg = container.querySelector('svg');
    expect(svg).not.toBeNull();
    expect(svg?.querySelectorAll('circle, ellipse, path')).toHaveLength(0);
    expect(svg?.innerHTML).not.toMatch(/orange|apple|leaf/);
    expect(svg?.querySelectorAll('.fill-brand-gold').length).toBe(1);
    expect(svg?.querySelectorAll('.fill-brand-gold-dark')).toHaveLength(2);
    expect(svg?.querySelector('linearGradient')).not.toBeNull();
  });

  it('only decorates: hidden from a screen reader, with no words in it', () => {
    const { container } = render(<BrandMark size={size} />);
    const svg = container.querySelector('svg');
    expect(svg?.getAttribute('aria-hidden')).toBe('true');
    expect(svg?.textContent).toBe('');
  });
});
