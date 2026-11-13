// The Firm tab in cards (plan 75, item 6): the names, the phone, address and trade mark, and the people
// of the print header each have a titled card, and the fields stay in reading order, so Enter goes from
// one to the next the way the eye does, row by row from the start edge, in Urdu and in English. The
// two names of a side stand beside each other from 480 px of free width (`@form2:grid-cols-2`); that
// the arrow keys follow the real positions is the job of `pickNeighbour` (tests/unit/form-navigation).

import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as settingsApi from '../../src/features/settings/api';
import * as urduApi from '../../src/shared/urdu/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { message, visible } from '../helpers/render';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the Firm tab in cards (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const label = (id: string) => (name: string) => visible(name) === message(language, id);

  async function open() {
    await renderApp('/settings');
    await screen.findByRole('heading', { level: 1, name: message(language, 'settings-title') });
    return userEvent.setup();
  }

  it('has a titled card for the names, for the phone, address and trade mark, and for the people', async () => {
    await open();
    for (const id of [
      'settings-firm-card-names',
      'settings-firm-card-details',
      'settings-firm-contacts-title',
    ]) {
      const heading = await screen.findByRole('heading', { level: 2, name: label(id) });
      expect(heading.closest('section')).toBeInTheDocument();
    }
  });

  it('puts the two names of a side in one two-column grid, the name typed first in the first cell', async () => {
    await open();
    const first = language === 'ur' ? 'settings-firm-name-ur' : 'settings-firm-name-en';
    const second = language === 'ur' ? 'settings-firm-name-en' : 'settings-firm-name-ur';
    const a = await screen.findByLabelText(label(first));
    const b = screen.getByLabelText(label(second));
    const grid = a.closest('.grid');
    expect(grid).toHaveClass('@form2:grid-cols-2');
    expect(grid).toContainElement(b);
    // The grid is made of the cells of a field: the first cell comes first in the document, so first in
    // the reading order of the screen, whatever the direction.
    const cells = Array.from(grid?.children ?? []);
    expect(cells.findIndex((cell) => cell.contains(a))).toBeLessThan(
      cells.findIndex((cell) => cell.contains(b)),
    );
  });

  it('Enter goes through the fields in reading order: the names, then phone, address, trade mark, then the people', async () => {
    const user = await open();
    const nameFirst = language === 'ur' ? 'settings-firm-name-ur' : 'settings-firm-name-en';
    const start = await screen.findByLabelText<HTMLInputElement>(label(nameFirst));
    start.focus();
    const order: string[] = [];
    for (let step = 0; step < 7; step += 1) {
      const focused = document.activeElement;
      order.push(
        focused instanceof HTMLElement
          ? visible(
              screen
                .getAllByText((_content, element) => element?.tagName === 'LABEL')
                .find((text) => text.getAttribute('for') === focused.id)?.textContent,
            )
          : '',
      );
      // Enter in the last field would save the form, so it is pressed between the fields only.
      if (step < 6) await user.keyboard('{Enter}');
    }
    const ids =
      language === 'ur'
        ? [
            'settings-firm-name-ur',
            'settings-firm-name-en',
            'settings-firm-mandi-ur',
            'settings-firm-mandi-en',
            'settings-firm-phone',
            'settings-firm-address',
            'settings-firm-trade-mark',
          ]
        : [
            'settings-firm-name-en',
            'settings-firm-name-ur',
            'settings-firm-mandi-en',
            'settings-firm-mandi-ur',
            'settings-firm-phone',
            'settings-firm-address',
            'settings-firm-trade-mark',
          ];
    expect(order).toEqual(ids.map((id) => message(language, id)));
    // The people of the header come after the trade mark, in the card of their own.
    const people = screen.getByRole('heading', {
      level: 2,
      name: label('settings-firm-contacts-title'),
    });
    const card = people.closest('section');
    if (card === null) throw new Error('the people of the header are not in a card');
    expect(
      within(card).getByRole('button', {
        name: label('settings-firm-contact-add'),
      }),
    ).toBeInTheDocument();
    expect(settingsApi.settingsSetFirm).not.toHaveBeenCalled();
  });
});
