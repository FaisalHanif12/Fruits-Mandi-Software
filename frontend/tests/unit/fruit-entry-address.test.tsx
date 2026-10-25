// The پھل screen opened by the address of one entry (plan 17d follow-up, PRD FR-19.4: the register
// opens a row to see its source), in the Urdu and the English app: `/rokar/fruit?entry=1` shows that
// entry in its saved view (what it holds as the backend has it, its parts, Edit and Cancel) and
// selects it in the list; an id the backend does not know, or text that is not an id, says the
// entry was not found in the normal words and leaves the form usable; Close goes back to the plain
// screen. The saved entry is the backend's own real answer (tests/fixtures/fruit_get_home.json).

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as fruitApi from '../../src/features/fruit/api';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { named } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeFruit, fakeFruitPickers } from '../helpers/fruit';
import * as real from '../helpers/fruit-real';
import { EMPTY_PAIRS } from '../helpers/paged';
import { fakePartners, THREE_PARTNERS } from '../helpers/partners';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/fruit/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners(THREE_PARTNERS);
  fakeFruitPickers();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

const said = (text: string) => (content: string) => visible(content) === text;

describe.each(LANGUAGES)('the fruit screen opened by an address (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const pageText = () => visible(document.body.textContent);
  const startsWith = (id: string) => (name: string) =>
    visible(name).startsWith(message(language, id));
  const fruitNumber = (number: number) => message(language, 'fruit-number', { number });
  const open = async (search: string) => {
    const view = await renderApp(`/rokar/fruit${search}`);
    await screen.findByRole('heading', { level: 1, name: named(message(language, 'fruit-title')) });
    return { ...view, user: userEvent.setup() };
  };
  const titleOfEntry = (number: number) =>
    said(message(language, 'fruit-entry-title', { number: fruitNumber(number) }));

  it('an id in the address opens that entry in its saved view and selects it in the list', async () => {
    const fake = fakeFruit();
    fake.rows.push({ ...real.FRUIT_GET_HOME });
    await open('?entry=1');
    const heading = await screen.findByText(titleOfEntry(1));
    const card = heading.closest('section');
    if (card === null) throw new Error('no entry card');
    expect(fruitApi.fruitGet).toHaveBeenCalledWith({ id: 1 });
    // What the backend holds now: the partner, the day, the lot, the item, the quantity, the rate,
    // the value and the part that comes off A.
    const text = visible(card.textContent);
    expect(text).toContain(language === 'ur' ? 'الف' : 'Partner A');
    expect(text).toContain('10-10-2026');
    expect(text).toContain('1-1');
    expect(text).toContain(language === 'ur' ? 'کیلا' : 'Banana');
    expect(text).toContain('580');
    expect(text).toContain('1,160');
    // The row is selected in the list below: its Edit and Cancel are on.
    await waitFor(() => {
      const edits = screen.getAllByRole('button', { name: startsWith('fruit-edit') });
      expect(edits).toHaveLength(2);
      for (const button of edits) expect(button).toBeEnabled();
    });
  });

  it('Edit in the saved view opens the form with the entry', async () => {
    const fake = fakeFruit();
    fake.rows.push({ ...real.FRUIT_GET_HOME });
    const { user } = await open('?entry=1');
    const card = (await screen.findByText(titleOfEntry(1))).closest('section');
    if (card === null) throw new Error('no entry card');
    await user.click(within(card).getByRole('button', { name: startsWith('fruit-edit') }));
    await screen.findByText(said(message(language, 'fruit-editing', { number: fruitNumber(1) })));
    expect(fruitApi.fruitGet).toHaveBeenCalledTimes(2);
  });

  it('a cancelled entry says so with its reason, and cannot be changed', async () => {
    const fake = fakeFruit();
    fake.rows.push({ ...real.FRUIT_GET_HOME_CANCELLED });
    await open('?entry=1');
    const card = (await screen.findByText(titleOfEntry(1))).closest('section');
    if (card === null) throw new Error('no entry card');
    expect(visible(card.textContent)).toContain(
      message(language, 'fruit-entry-cancelled', { reason: 'not taken' }),
    );
    expect(within(card).getByRole('button', { name: startsWith('fruit-edit') })).toBeDisabled();
    expect(within(card).getByRole('button', { name: startsWith('fruit-cancel') })).toBeDisabled();
  });

  it('an id the backend does not know says not found in the normal words, and the form still works', async () => {
    fakeFruit();
    await open('?entry=99');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'err-not-found'));
    });
    expect(fruitApi.fruitGet).toHaveBeenCalledWith({ id: 99 });
    expect(pageText()).toContain(message(language, 'fruit-entry-failed'));
    // The form is there and takes typing.
    const user = userEvent.setup();
    const qty = screen.getByRole('textbox', { name: named(message(language, 'fruit-field-qty')) });
    await user.type(qty, '2');
    expect(qty).toHaveValue('2');
  });

  it('text that is not an id says the same, and nothing is asked of the backend for it', async () => {
    fakeFruit();
    await open('?entry=abc');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'err-not-found'));
    });
    expect(fruitApi.fruitGet).not.toHaveBeenCalled();
  });

  it('Close goes back to the plain screen', async () => {
    const fake = fakeFruit();
    fake.rows.push({ ...real.FRUIT_GET_HOME });
    const { user, router } = await open('?entry=1');
    const card = (await screen.findByText(titleOfEntry(1))).closest('section');
    if (card === null) throw new Error('no entry card');
    await user.click(within(card).getByRole('button', { name: startsWith('fruit-entry-close') }));
    await waitFor(() => {
      expect(screen.queryByText(titleOfEntry(1))).toBeNull();
    });
    expect(router.state.location.search).toEqual({});
  });
});
