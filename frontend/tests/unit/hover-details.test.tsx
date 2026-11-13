// HoverDetails (plan 75 T6, PRD FR-19.14): the card that opens beside a control on hover and on a
// key's focus. It opens after a short wait under the pointer and at once for the keyboard, closes on
// leave and on Esc, opens on the end side of the text in each direction, is one at a time, and a long
// press opens it on a touch screen. Urdu and English.
//
// The words in the test are real messages of the app (the lint rule keeps text out of code, also here):
// the first control's card is named by `reports-title` and says `reports-help`; the second control's
// card is named by `reports-search-label` and says `reports-jump-label`.

import { Localized, useLocalization } from '@fluent/react';
import { act, fireEvent, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import {
  HOVER_OPEN_DELAY_MS,
  HoverDetails,
  LONG_PRESS_MS,
  endSideOf,
  focusQuietly,
} from '../../src/shared/ui/HoverDetails';
import { message, renderIn, visible } from '../helpers/render';

function Subject() {
  const { l10n } = useLocalization();
  return (
    <>
      <HoverDetails
        label={l10n.getString('reports-title')}
        card={
          <p>
            <Localized id="reports-help" />
          </p>
        }
      >
        <a href="#one" data-testid="one">
          <Localized id="reports-search-clear" />
        </a>
      </HoverDetails>
      <HoverDetails
        label={l10n.getString('reports-search-label')}
        card={
          <p>
            <Localized id="reports-jump-label" />
          </p>
        }
      >
        <a href="#two" data-testid="two">
          <Localized id="reports-search-none" />
        </a>
      </HoverDetails>
    </>
  );
}

const SLOW_MS = { timeout: 5_000 };

// The side is a pure choice. The browser places the card (and flips it near an edge); jsdom has no
// layout, so the side it ends on is checked by the screenshots, not here.
describe('the side a card opens on', () => {
  it('is the end of the text: right in English, left in Urdu', () => {
    expect(endSideOf('ltr')).toBe('right');
    expect(endSideOf('rtl')).toBe('left');
  });
});

describe.each(LANGUAGES)('HoverDetails (%s)', (language: Language) => {
  const m = (id: string) => message(language, id);
  const ONE = 'reports-title';
  const TWO = 'reports-search-label';

  const open = () => {
    renderIn(language, <Subject />);
    return {
      user: userEvent.setup(),
      one: screen.getByTestId('one'),
      two: screen.getByTestId('two'),
    };
  };
  /** The card named by a message, or null when it is not open. */
  const cardOf = (id: string) =>
    screen.queryByRole('group', { name: (name) => visible(name) === m(id) });

  it('opens a moment after the pointer arrives, not at once', async () => {
    const { user, one } = open();
    await user.hover(one);
    expect(cardOf(ONE)).toBeNull();
    const card = await screen.findByRole(
      'group',
      { name: (name) => visible(name) === m(ONE) },
      SLOW_MS,
    );
    expect(visible(card.textContent)).toBe(m('reports-help'));
  });

  it('closes a moment after the pointer leaves', async () => {
    const { user, one } = open();
    await user.hover(one);
    await screen.findByRole('group', { name: (name) => visible(name) === m(ONE) }, SLOW_MS);
    await user.unhover(one);
    await waitFor(() => {
      expect(cardOf(ONE)).toBeNull();
    }, SLOW_MS);
  });

  it('opens at once when the control gets the focus from a key, and closes when the focus leaves', async () => {
    const { user, one } = open();
    await user.tab();
    expect(one).toHaveFocus();
    expect(visible(cardOf(ONE)?.textContent)).toBe(m('reports-help'));
    expect(one.getAttribute('aria-describedby')).not.toBeNull();
    await user.tab();
    await waitFor(() => {
      expect(cardOf(ONE)).toBeNull();
    }, SLOW_MS);
  });

  it('does not open for a focus that a click made, or that nothing but a script made', async () => {
    const { user, one, two } = open();
    await user.click(one);
    expect(one).toHaveFocus();
    expect(cardOf(ONE)).toBeNull();
    // The focus is moved by a script after a click (a screen that focuses its first item as it opens).
    act(() => {
      two.focus();
    });
    await new Promise((done) => setTimeout(done, HOVER_OPEN_DELAY_MS + 100));
    expect(cardOf(TWO)).toBeNull();
  });

  it('does not open for a focus a screen sets on purpose with focusQuietly, even right after a key', async () => {
    const { user, one } = open();
    await user.tab();
    await user.tab();
    act(() => {
      focusQuietly(one);
    });
    await new Promise((done) => setTimeout(done, HOVER_OPEN_DELAY_MS + 100));
    expect(cardOf(ONE)).toBeNull();
    expect(cardOf(TWO)).toBeNull();
  });

  it('opens for a focus that a script moves after a key was pressed (the arrow keys of a list)', async () => {
    const { user, two } = open();
    await user.tab();
    await user.keyboard('{ArrowDown}');
    act(() => {
      two.focus();
    });
    expect(cardOf(TWO)).not.toBeNull();
    expect(cardOf(ONE)).toBeNull();
  });

  it('closes on Esc and keeps the focus where it was', async () => {
    const { user, one } = open();
    await user.tab();
    expect(cardOf(ONE)).not.toBeNull();
    await user.keyboard('{Escape}');
    expect(cardOf(ONE)).toBeNull();
    expect(one).toHaveFocus();
  });

  it('shows one card at a time', async () => {
    const { user, two } = open();
    await user.tab();
    expect(cardOf(ONE)).not.toBeNull();
    await user.hover(two);
    await screen.findByRole('group', { name: (name) => visible(name) === m(TWO) }, SLOW_MS);
    expect(cardOf(ONE)).toBeNull();
    expect(screen.getAllByRole('group')).toHaveLength(1);
  });

  it('carries the language and direction of the page', async () => {
    const { user } = open();
    await user.tab();
    const card = cardOf(ONE);
    expect(card).toHaveAttribute('lang', language);
    expect(card).toHaveAttribute('dir', language === 'ur' ? 'rtl' : 'ltr');
  });

  it('opens on a long press of a finger, and the tap that ends it does not follow the link', async () => {
    const { one } = open();
    fireEvent.pointerDown(one, { pointerType: 'touch' });
    expect(cardOf(ONE)).toBeNull();
    await act(async () => {
      await new Promise((done) => setTimeout(done, LONG_PRESS_MS + 100));
    });
    expect(cardOf(ONE)).not.toBeNull();
    fireEvent.pointerUp(one, { pointerType: 'touch' });
    // fireEvent returns false when the click was stopped from doing its default.
    expect(fireEvent.click(one)).toBe(false);
    // A second tap is a plain tap: the link is followed.
    expect(fireEvent.click(one)).toBe(true);
  });

  it('does not open on a short touch', async () => {
    const { one } = open();
    fireEvent.pointerDown(one, { pointerType: 'touch' });
    fireEvent.pointerUp(one, { pointerType: 'touch' });
    await act(async () => {
      await new Promise((done) => setTimeout(done, LONG_PRESS_MS + 100));
    });
    expect(cardOf(ONE)).toBeNull();
  });
});
