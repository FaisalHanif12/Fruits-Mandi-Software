// The mark of the app (plan 75, plan 93 T2 and T3, Umar 22-11-2026): the app icon itself, the navy tile with the gold crate,
// an orange, an apple and a leaf, as a bundled picture (an inline drawing drew the fruit black). It stands at the top of the main menu only. The sign-in panel has no logo at
// all (see auth-brand-panel.test.tsx). The mark is the same in both languages, and a screen reader skips it.

import { readFileSync } from 'node:fs';
import path from 'node:path';
import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { MainNav } from '../../src/app/layout/MainNav';
import { LANGUAGES } from '../../src/i18n';
import { BrandMark } from '../../src/shared/ui/BrandMark';
import { renderIn } from '../helpers/render';

describe('the brand mark', () => {
  it('is the real app icon as a bundled picture: the fruit and the crate on the navy tile, no network', () => {
    const { container } = render(<BrandMark />);
    const picture = container.querySelector('img');
    expect(picture).not.toBeNull();
    // A file of the app, never a web address (the Content Security Policy allows only the app's own files).
    expect(picture?.getAttribute('src')).toMatch(/app-icon.*\.png$/);
    expect(picture?.getAttribute('src')).not.toMatch(/^https?:/);
    // It is the 256 px icon of the app (backend/app/icons/128x128@2x.png), copied byte for byte.
    const copy = readFileSync(path.resolve(__dirname, '../../src/assets/app-icon.png'));
    const original = readFileSync(
      path.resolve(__dirname, '../../../backend/app/icons/128x128@2x.png'),
    );
    expect(copy.equals(original)).toBe(true);
  });

  it('only decorates: hidden from a screen reader, no words, an empty alt', () => {
    const { container } = render(<BrandMark />);
    const box = container.firstElementChild;
    expect(box?.getAttribute('aria-hidden')).toBe('true');
    expect(box?.textContent).toBe('');
    expect(container.querySelector('img')?.getAttribute('alt')).toBe('');
  });

  it('is drawn a quarter larger inside a box that clips it, so the tile fills the box', () => {
    const { container } = render(<BrandMark />);
    expect(container.firstElementChild).toHaveClass('overflow-hidden', 'size-target');
    expect(container.querySelector('img')).toHaveClass('scale-125', 'size-full');
  });
});

describe.each(LANGUAGES)('the main menu (%s)', (language) => {
  it('starts with the fruit mark, which no screen reader reads', () => {
    renderIn(language, <MainNav current="dashboard" onSelect={vi.fn()} onSignOut={vi.fn()} />);
    const menu = screen.getByRole('navigation');
    const mark = menu.querySelector('img');
    expect(mark).not.toBeNull();
    expect(mark?.closest('[aria-hidden="true"]')).not.toBeNull();
  });
});
