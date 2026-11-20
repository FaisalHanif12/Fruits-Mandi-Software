// The box every dropdown list is drawn in (plan 86, NFR-USE-16, Code Standards F7): below its field when the
// whole list fits, above it when there is more room above, its height the room on that side, inside the
// window sideways, clear of the bars of the shell, in a portal so a card or a grid with `overflow:
// hidden` never cuts it, and following its field when the page scrolls. Urdu and English. jsdom has no
// layout, so the sizes are given (tests/helpers/dropdown-layout.ts); the rules themselves are in
// dropdown-position.test.ts.

import { act, cleanup, screen } from '@testing-library/react';
import { useRef } from 'react';
import { afterEach, describe, expect, it } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import { DropdownPopup } from '../../src/shared/ui';
import { mockDropdownLayout, type DropdownLayout, type RectSpec } from '../helpers/dropdown-layout';
import { message, renderIn } from '../helpers/render';

/** The window the app is built for at its smallest comfortable size: 1366 x 768, a 32 px status bar. */
const WINDOW = { width: 1366, height: 768 };
const STATUS_BAR: RectSpec = { top: 736, left: 0, width: 1366, height: 32 };
const FIELD_SIZE = { width: 280, height: 44 };
/** A list of 12 rows: taller than the room on either side in a short window. */
const LONG_LIST = { width: 280, height: 700 };
const SHORT_LIST = { width: 280, height: 200 };

/** Any words of the app will do for a row: the test is about where the list goes. */
const ROW_WORDS = message('en', 'search-list-label');

function Rig({ open = true, prefer }: { open?: boolean; prefer?: 'top' | 'bottom' }) {
  const field = useRef<HTMLDivElement>(null);
  return (
    <div>
      <footer data-testid="status-bar" data-dropdown-avoid="" />
      {/* A card that cuts what leaves it, like a grid cell or a card of the app. */}
      <div data-testid="card" style={{ overflow: 'hidden' }}>
        <div ref={field} data-testid="field" />
        <DropdownPopup open={open} anchorRef={field} prefer={prefer}>
          <ul role="listbox" aria-label={ROW_WORDS}>
            <li role="option" aria-selected="false">
              {ROW_WORDS}
            </li>
          </ul>
        </DropdownPopup>
      </div>
    </div>
  );
}

let layout: DropdownLayout | null = null;
afterEach(() => {
  layout?.restore();
  layout = null;
});

/** Draws the rig with the field at `at` and a list of the given size. */
function draw(
  language: (typeof LANGUAGES)[number],
  at: Partial<RectSpec>,
  list: { width: number; height: number },
  props: { open?: boolean; prefer?: 'top' | 'bottom' } = {},
) {
  layout = mockDropdownLayout({
    window: WINDOW,
    field: () => screen.queryByTestId('field'),
    fieldRect: { top: 100, left: 300, ...FIELD_SIZE, ...at },
    list,
    bars: { 'status-bar': STATUS_BAR },
  });
  renderIn(language, <Rig {...props} />);
  const popup = document.querySelector<HTMLElement>('[data-dropdown-popup]');
  if (popup === null) throw new Error('the dropdown box is not on the page');
  return popup;
}

describe.each(LANGUAGES)('the dropdown box (%s)', (language) => {
  it('opens below a field near the top when the whole list fits', () => {
    const popup = draw(language, { top: 100 }, SHORT_LIST);
    expect(popup).toHaveAttribute('data-side', 'bottom');
    // field bottom 144 and the 4 px gap
    expect(popup.style.insetBlockStart).toBe('148px');
    expect(popup.style.insetBlockEnd).toBe('auto');
  });

  it('flips above a field near the bottom of the window, tied to the field', () => {
    const popup = draw(language, { top: 600 }, SHORT_LIST);
    expect(popup).toHaveAttribute('data-side', 'top');
    // 768 - (600 - 4): the bottom edge of the list is the field's top edge less the gap
    expect(popup.style.insetBlockEnd).toBe('172px');
    expect(popup.style.insetBlockStart).toBe('auto');
  });

  it('cuts the height to the room on the side it uses, clear of the status bar and the edges', () => {
    const below = draw(language, { top: 100 }, LONG_LIST);
    // below: the status bar starts at 736, 8 px edge, field bottom 144 and a 4 px gap
    expect(below).toHaveAttribute('data-side', 'bottom');
    expect(below.style.maxHeight).toBe(`${String(736 - 8 - 148)}px`);
    layout?.restore();
    cleanup();

    const above = draw(language, { top: 600 }, LONG_LIST);
    // above: nothing on top in this rig; 8 px edge, field top 600 and a 4 px gap
    expect(above).toHaveAttribute('data-side', 'top');
    expect(above.style.maxHeight).toBe(`${String(600 - 4 - 8)}px`);
  });

  it('keeps below when above has no more room, with its height cut to the room below', () => {
    const popup = draw(language, { top: 300 }, LONG_LIST);
    // 736 - 8 - 348 = 380 below, 288 above: below, and cut
    expect(popup).toHaveAttribute('data-side', 'bottom');
    expect(popup.style.maxHeight).toBe('380px');
  });

  it('prefers the side it is told to when the list fits there', () => {
    const popup = draw(language, { top: 300 }, SHORT_LIST, { prefer: 'top' });
    expect(popup).toHaveAttribute('data-side', 'top');
  });

  it('stays inside the window sideways, in both directions', () => {
    // A field at the end of the window, a list wider than the field.
    const wide = { width: 420, height: 200 };
    const atEnd = draw(language, { top: 100, left: 1200, width: 150 }, wide);
    const rtl = language === 'ur';
    const inlineStart = Number.parseFloat(atEnd.style.insetInlineStart);
    // In Urdu the start edge is the right edge of the window, so the offset is measured from there.
    if (rtl) {
      // the field's right edge is at 1350, so the list starts 16 px from the right edge: inside
      expect(inlineStart).toBe(WINDOW.width - 1350);
    } else {
      // pushed back so that its end edge is 8 px inside the window: 1366 - 8 - 420
      expect(inlineStart).toBe(WINDOW.width - 8 - 420);
    }
    expect(atEnd.dataset['side']).toBe('bottom');
    layout?.restore();
    cleanup();

    // A field at the other end.
    const atStart = draw(language, { top: 100, left: 10, width: 150 }, wide);
    const offset = Number.parseFloat(atStart.style.insetInlineStart);
    if (rtl) expect(offset).toBe(WINDOW.width - (8 + 420));
    else expect(offset).toBe(10);
  });

  it('is drawn in a portal on the page, so a card that cuts what leaves it does not cut it', () => {
    const popup = draw(language, { top: 600 }, SHORT_LIST);
    const card = screen.getByTestId('card');
    expect(card.contains(popup)).toBe(false);
    expect(popup.parentElement).toBe(document.body);
    // and it is a fixed box, which an `overflow: hidden` ancestor cannot cut
    expect(popup).toHaveClass('fixed');
    expect(screen.getByRole('listbox')).toBeInTheDocument();
  });

  it('carries the language and the direction of the app, being outside the page', () => {
    const popup = draw(language, { top: 100 }, SHORT_LIST);
    expect(popup).toHaveAttribute('lang', language);
    expect(popup).toHaveAttribute('dir', language === 'ur' ? 'rtl' : 'ltr');
  });

  it('follows its field when the page scrolls, and flips when the field has moved down', () => {
    const popup = draw(language, { top: 100 }, SHORT_LIST);
    expect(popup).toHaveAttribute('data-side', 'bottom');
    act(() => {
      layout?.moveField({ top: 620 });
      window.dispatchEvent(new Event('scroll'));
    });
    expect(popup).toHaveAttribute('data-side', 'top');
    expect(popup.style.insetBlockEnd).toBe(`${String(768 - (620 - 4))}px`);
    act(() => {
      layout?.moveField({ top: 150 });
      window.dispatchEvent(new Event('resize'));
    });
    expect(popup).toHaveAttribute('data-side', 'bottom');
    expect(popup.style.insetBlockStart).toBe('198px');
  });

  it('takes no room while closed', () => {
    const popup = draw(language, { top: 100 }, SHORT_LIST, { open: false });
    expect(popup).toHaveClass('hidden');
    expect(popup).not.toHaveClass('fixed');
  });
});
