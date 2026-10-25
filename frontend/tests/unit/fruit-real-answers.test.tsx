// The fruit screen with the backend's REAL answers (plan 17d, T2; tests/fixtures/fruit_*.json, copied,
// never edited), in the Urdu and the English app, with the numbers of E2E-42: home fruit 2 × 580 =
// 1,160 off A and the lot from 5 to 3; guest fruit of 3,000 shared 1,000 each; a rupee that does not
// divide goes to the partner made first; an item sold by weight; the refusals at their fields; the
// edit that gives the lot its quantity back; the cancel and its refusal. Each save, edit and cancel
// sends exactly the request the backend's own test sent. Every figure on the screen is a figure the
// backend sent; nothing is worked out here (CLAUDE.md rule 1).

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as fruitApi from '../../src/features/fruit/api';
import { LANGUAGES } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import * as searchApi from '../../src/shared/search/api';
import * as urduApi from '../../src/shared/urdu/api';
import { account, fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
import { named } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import * as real from '../helpers/fruit-real';
import { EMPTY_PAIRS } from '../helpers/paged';
import { fakePartners, THREE_PARTNERS } from '../helpers/partners';
import { settlePreview } from '../helpers/preview-settled';
import { message, visible } from '../helpers/render';
import { sendsTheSame } from '../helpers/shapes';

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

const HAJI = account(31, 'seller', '030110003', 'حاجی نذیر', 'Haji Nazeer');

/** The masters the backend's test used: item 2 (apple), item 5 (sold by weight), packing 1 (box). */
const hit = (id: number, kind: 'item' | 'packing', ur: string, en: string) => ({
  id,
  kind,
  code: '',
  name_ur: ur,
  name_en: en,
  extra_ur: '',
  extra_en: '',
  is_active: true,
});
const masters = (scope: string) =>
  scope === 'item'
    ? [hit(2, 'item', 'سیب', 'Apple'), hit(5, 'item', 'آم', 'Mango')]
    : scope === 'packing'
      ? [hit(1, 'packing', 'پیٹی', 'Box')]
      : [];

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners(THREE_PARTNERS);
  fakeAccounts([HAJI]);
  fakeAccountSearch([HAJI]);
  vi.mocked(searchApi.searchMasters).mockImplementation(({ scope }) =>
    Promise.resolve(masters(scope)),
  );
  vi.mocked(searchApi.searchRecent).mockImplementation(({ scope }) =>
    Promise.resolve(masters(scope)),
  );
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(fruitApi.fruitLots).mockResolvedValue(real.FRUIT_LOTS_ALL);
  vi.mocked(fruitApi.fruitList).mockResolvedValue(real.FRUIT_LIST_ALL);
});

afterEach(() => {
  leaveAppWindow();
});

const said = (text: string) => (content: string) => visible(content) === text;
const pageText = () => visible(document.body.textContent);
type User = ReturnType<typeof userEvent.setup>;

describe.each(LANGUAGES)('the fruit screen with the backend’s real answers (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const partnerA = language === 'ur' ? 'الف' : 'Partner A';
  const startsWith = (id: string) => (name: string) =>
    visible(name).startsWith(message(language, id));
  const fruitNumber = (number: number) => message(language, 'fruit-number', { number });

  const open = async () => {
    const view = await renderApp('/rokar/fruit');
    await screen.findByRole('heading', { level: 1, name: named(message(language, 'fruit-title')) });
    return { ...view, user: userEvent.setup() };
  };
  const field = (id: string) => {
    const label = named(message(language, id));
    return (
      screen.queryByRole('textbox', { name: label }) ??
      screen.getByRole('combobox', { name: label })
    );
  };
  const choose = async (user: User, id: string, optionText: string) => {
    await user.click(field(id));
    await user.click(await screen.findByRole('option', { name: named(optionText) }));
  };
  const pickLot = async (user: User) => {
    const box = field('fruit-field-lot');
    await user.click(box);
    await user.type(box, 'rahim');
    await user.click(await screen.findByRole('option'));
    await waitFor(() => {
      expect(box).toHaveValue(language === 'ur' ? 'کیلا پیٹی' : 'Banana Box');
    });
  };
  /** Takes the first match of a picker field after typing some letters. */
  const pickFirst = async (user: User, id: string, text: string, shown?: RegExp) => {
    const box = field(id);
    await user.click(box);
    await user.type(box, text);
    await user.click(
      shown === undefined
        ? await screen.findByRole('option')
        : await screen.findByRole('option', { name: (name) => shown.test(visible(name)) }),
    );
    // The pick has landed when the field shows the name instead of the letters typed.
    await waitFor(() => {
      expect(box).not.toHaveValue(text);
    });
  };
  async function previewSettled(rate: string) {
    await waitFor(() => {
      expect(fruitApi.fruitPreview).toHaveBeenLastCalledWith(expect.objectContaining({ rate }));
    });
    await settlePreview();
  }
  const info = (label: string) => {
    const term = screen.getAllByText(said(label))[0];
    return within(term?.closest('div') ?? document.body).getByRole('button', {
      name: startsWith('calc-info-label'),
    });
  };

  /** Home fruit of E2E-42 typed on a fresh form: A, the lot of رحیم, 2 at 580. */
  async function typeHome(user: User) {
    await choose(user, 'fruit-field-for', partnerA);
    await pickLot(user);
    await user.type(field('fruit-field-qty'), '2');
    // The rate field is still empty here: the last rate shows only as grey text (PRD FR-7.53).
    await waitFor(() => {
      expect(field('fruit-field-rate')).toHaveAttribute('placeholder', '580');
    });
    expect(field('fruit-field-rate')).toHaveValue('');
    expect(pageText()).toContain(message(language, 'bikri-rate-hint-today', { rate: '580' }));
    await user.type(field('fruit-field-rate'), '580');
    await previewSettled('580');
  }
  /** Guest fruit from another shop, as the backend's test typed it. */
  const APPLE = { text: 'app', shown: /سیب|Apple/ };
  const MANGO = { text: 'mango', shown: /آم|Mango/ };
  async function typeGuests(user: User, item: typeof APPLE, rate: string, weight?: string) {
    await choose(user, 'fruit-field-for', message(language, 'fruit-for-guests'));
    await user.click(
      within(
        screen.getByRole('group', { name: named(message(language, 'fruit-field-source')) }),
      ).getByRole('button', { name: named(message(language, 'fruit-source-other')) }),
    );
    await pickFirst(user, 'fruit-field-agent', 'nazir');
    await pickFirst(user, 'glossary-qism-maal', item.text, item.shown);
    await pickFirst(user, 'glossary-packing', 'box');
    await user.type(field('fruit-field-qty'), '3');
    if (weight !== undefined) await user.type(field('fruit-field-weight'), weight);
    await user.type(field('fruit-field-rate'), rate);
  }

  it('the lot picker lists the real lot with what is left', async () => {
    const { user } = await open();
    await user.click(field('fruit-field-lot'));
    const option = await screen.findByRole('option');
    const text = visible(option.textContent);
    expect(text).toContain(language === 'ur' ? 'رحیم' : 'Rahim');
    expect(text).toContain('1-1');
    expect(text).toContain('5');
  });

  it('home fruit (E2E-42): the real figures show, the rate stays empty, and F10 sends the backend’s own request', async () => {
    vi.mocked(fruitApi.fruitPreview).mockResolvedValue(real.FRUIT_PREVIEW_HOME);
    vi.mocked(fruitApi.fruitSave).mockResolvedValue(real.FRUIT_SAVE_HOME);
    const { user } = await open();
    await typeHome(user);
    await waitFor(() => {
      const text = pageText();
      // The value, A's total now and after, the lot before and after.
      for (const figure of ['1,160', '1,424', '264']) expect(text, figure).toContain(figure);
    });
    expect(pageText()).toContain(language === 'ur' ? 'رحیم' : 'Rahim');
    // The working of the value is the backend's, written with its message.
    await user.click(info(message(language, 'fruit-numbers-value')));
    const working = await screen.findByRole('dialog');
    expect(visible(working.textContent)).toContain('580');
    expect(visible(working.textContent)).toContain('1,160');
    expect(visible(working.textContent)).not.toContain('working-fruit-value');
    await user.keyboard('{Escape}');
    press('F10');
    await waitFor(() => {
      expect(fruitApi.fruitSave).toHaveBeenCalledTimes(1);
    });
    const sent = vi.mocked(fruitApi.fruitSave).mock.calls[0]?.[0];
    if (sent === undefined) throw new Error('nothing sent');
    expect(sendsTheSame(sent, real.FRUIT_SAVE_HOME_REQUEST)).toBeNull();
    await screen.findByText(said(message(language, 'fruit-saved', { number: fruitNumber(1) })));
  });

  it('guest fruit of 3,000 from another shop is shared 1,000 each, with the backend’s working', async () => {
    vi.mocked(fruitApi.fruitPreview).mockResolvedValue(real.FRUIT_PREVIEW_GUESTS);
    vi.mocked(fruitApi.fruitSave).mockResolvedValue(real.FRUIT_SAVE_GUESTS);
    const { user } = await open();
    await typeGuests(user, APPLE, '1,000');
    await previewSettled('1,000');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'fruit-parts-title-guests'));
    });
    const table = screen.getAllByRole('table')[0];
    if (table === undefined) throw new Error('no parts table');
    const rows = within(table).getAllByRole('row').slice(1);
    expect(rows).toHaveLength(3);
    // His part, his total now and after, as the backend sent them (A: 264 becomes -736).
    expect(visible(rows[0]?.textContent)).toContain('1,000');
    expect(visible(rows[0]?.textContent)).toContain('264');
    expect(visible(rows[0]?.textContent)).toContain('-736');
    expect(pageText()).toContain('3,000');
    await user.click(info(message(language, 'fruit-parts-title-guests')));
    const working = await screen.findByRole('dialog');
    expect(visible(working.textContent)).not.toContain('working-fruit-guests');
    expect(visible(working.textContent)).toContain('3,000');
    await user.keyboard('{Escape}');
    press('F10');
    await waitFor(() => {
      expect(fruitApi.fruitSave).toHaveBeenCalledTimes(1);
    });
    const sent = vi.mocked(fruitApi.fruitSave).mock.calls[0]?.[0];
    if (sent === undefined) throw new Error('nothing sent');
    expect(sendsTheSame(sent, real.FRUIT_SAVE_GUESTS_REQUEST)).toBeNull();
  });

  it('a rupee that does not divide goes to the partner made first: one part, with the note', async () => {
    vi.mocked(fruitApi.fruitPreview).mockResolvedValue(real.FRUIT_PREVIEW_GUESTS_ONE_RUPEE);
    const { user } = await open();
    await typeGuests(user, APPLE, '1');
    await previewSettled('1');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'fruit-parts-title-guests'));
    });
    const table = screen.getAllByRole('table')[0];
    if (table === undefined) throw new Error('no parts table');
    expect(within(table).getAllByRole('row').slice(1)).toHaveLength(1);
    expect(pageText()).toContain(message(language, 'fruit-parts-note'));
  });

  it('an item sold by weight: 12.345 kg at 100 a kilo is 1,235, shared 413 / 411 / 411', async () => {
    vi.mocked(fruitApi.fruitPreview).mockResolvedValue(real.FRUIT_PREVIEW_WEIGHT_ITEM);
    const { user } = await open();
    await typeGuests(user, MANGO, '100', '12.345');
    await previewSettled('100');
    await waitFor(() => {
      expect(pageText()).toContain('1,235');
    });
    const table = screen.getAllByRole('table')[0];
    if (table === undefined) throw new Error('no parts table');
    const cells = within(table)
      .getAllByRole('row')
      .slice(1)
      .map((row) => visible(within(row).getAllByRole('cell')[0]?.textContent));
    expect(cells).toEqual(['413', '411', '411']);
    await user.click(info(message(language, 'fruit-numbers-value')));
    const working = await screen.findByRole('dialog');
    expect(visible(working.textContent)).toContain('12345');
    expect(visible(working.textContent)).not.toContain('working-fruit-value-weight');
  });

  it('a weight item with no weight is told at the weight field', async () => {
    vi.mocked(fruitApi.fruitPreview).mockResolvedValue(real.FRUIT_PREVIEW_WEIGHT_REQUIRED);
    const { user } = await open();
    await typeGuests(user, MANGO, '100');
    await previewSettled('100');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'err-weight-required'));
    });
  });

  it('more than the lot has and a rate of 0 are told at their fields, with what is left', async () => {
    vi.mocked(fruitApi.fruitPreview).mockResolvedValue(real.FRUIT_PREVIEW_QTY_EXCEEDS_LOT);
    const { user } = await open();
    await choose(user, 'fruit-field-for', partnerA);
    await pickLot(user);
    await user.type(field('fruit-field-qty'), '9');
    await user.type(field('fruit-field-rate'), '580');
    await previewSettled('580');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'err-qty-exceeds-lot', { available: '5' }));
    });
    vi.mocked(fruitApi.fruitPreview).mockResolvedValue(real.FRUIT_PREVIEW_RATE_ZERO);
    await user.clear(field('fruit-field-rate'));
    await user.type(field('fruit-field-rate'), '0');
    await previewSettled('0');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'err-must-be-more-than-zero'));
    });
  });

  it('a day before the goods came is refused at the day field, and nothing typed is lost', async () => {
    vi.mocked(fruitApi.fruitPreview).mockResolvedValue(real.FRUIT_PREVIEW_HOME);
    vi.mocked(fruitApi.fruitSave).mockRejectedValue(
      new CommandError(real.FRUIT_SAVE_DATE_BEFORE_RECEIVED),
    );
    const { user } = await open();
    await typeHome(user);
    press('F10');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'err-date-before-received'));
    });
    expect(field('fruit-field-qty')).toHaveValue('2');
    expect(field('fruit-field-rate')).toHaveValue('580');
  });

  it('the list shows the real entries and the totals the backend worked out over the whole filter', async () => {
    await open();
    const list = await screen.findByRole('group', {
      name: named(message(language, 'fruit-list-label')),
    });
    await waitFor(() => {
      expect(list.querySelectorAll('.ag-row[row-id]').length).toBeGreaterThan(1);
    });
    const text = visible(list.textContent);
    // 2 entries: 2 and 3 نگ; the values 1,160 and 3,000; the total row 5 and 4,160 (the backend's).
    for (const figure of ['1,160', '3,000', '4,160']) expect(text, figure).toContain(figure);
    const pinned = document.querySelector('.ag-row-pinned');
    expect(visible(pinned?.querySelector('[col-id="qty"]')?.textContent)).toBe('5');
    expect(visible(pinned?.querySelector('[col-id="value"]')?.textContent)).toBe('4,160');
  });

  it('the guests filter asks for guests only, and a cancelled entry says so with no total', async () => {
    const { user } = await open();
    await choose(user, 'fruit-filter-for', message(language, 'fruit-for-guests'));
    await waitFor(() => {
      expect(fruitApi.fruitList).toHaveBeenLastCalledWith(
        expect.objectContaining({ guests_only: true, partner_id: null }),
      );
    });
    vi.mocked(fruitApi.fruitList).mockResolvedValue(real.FRUIT_LIST_CANCELLED);
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'fruit-period-all')) }),
    );
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'fruit-for-cancelled', { name: partnerA }));
    });
    const pinned = document.querySelector('.ag-row-pinned');
    expect(visible(pinned?.querySelector('[col-id="value"]')?.textContent)).toBe('0');
  });

  it('edit: the form opens with the real entry, the preview carries its id, and the edit sends the backend’s own request', async () => {
    vi.mocked(fruitApi.fruitGet).mockResolvedValue(real.FRUIT_GET_HOME);
    vi.mocked(fruitApi.fruitPreview).mockResolvedValue(real.FRUIT_PREVIEW_EDIT_HOME);
    vi.mocked(fruitApi.fruitEdit).mockResolvedValue(real.FRUIT_EDIT_HOME);
    const { user } = await open();
    const list = await screen.findByRole('group', {
      name: named(message(language, 'fruit-list-label')),
    });
    const row = await waitFor(() => {
      const found = list.querySelector<HTMLElement>('.ag-row[row-id="1"] .ag-cell');
      if (found === null) throw new Error('no row');
      return found;
    });
    await user.click(row);
    await user.click(screen.getByRole('button', { name: startsWith('fruit-edit') }));
    await screen.findByText(said(message(language, 'fruit-editing', { number: fruitNumber(1) })));
    expect(fruitApi.fruitGet).toHaveBeenCalledWith({ id: 1 });
    expect(field('fruit-field-qty')).toHaveValue('2');
    expect(field('fruit-field-rate')).toHaveValue('580');
    expect(field('fruit-field-lot')).toHaveValue(language === 'ur' ? 'کیلا پیٹی' : 'Banana Box');
    await user.clear(field('fruit-field-qty'));
    await user.type(field('fruit-field-qty'), '3');
    await waitFor(() => {
      expect(fruitApi.fruitPreview).toHaveBeenLastCalledWith(
        expect.objectContaining({ qty: '3', editing_id: 1 }),
      );
    });
    const asked = vi.mocked(fruitApi.fruitPreview).mock.calls.at(-1)?.[0];
    if (asked === undefined) throw new Error('no preview');
    expect(sendsTheSame(asked, real.FRUIT_PREVIEW_EDIT_HOME_REQUEST)).toBeNull();
    // The lot has 5 again for this entry (its 2 given back) and A's total is the backend's.
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'fruit-qty-left', { remaining: '5' }));
      expect(pageText()).toContain('1,740');
    });
    await user.type(field('fruit-field-reason'), 'wrong quantity');
    await settlePreview();
    press('F10');
    await waitFor(() => {
      expect(fruitApi.fruitEdit).toHaveBeenCalledTimes(1);
    });
    const sent = vi.mocked(fruitApi.fruitEdit).mock.calls[0]?.[0];
    if (sent === undefined) throw new Error('nothing sent');
    expect(sendsTheSame(sent, real.FRUIT_EDIT_HOME_REQUEST)).toBeNull();
  });

  it('cancel: the box sends the backend’s own request; cancelling again is refused in words', async () => {
    vi.mocked(fruitApi.fruitList).mockResolvedValue({
      ...real.FRUIT_LIST_ALL,
      rows: [real.FRUIT_GET_HOME_EDITED],
    });
    vi.mocked(fruitApi.fruitCancel).mockResolvedValueOnce(real.FRUIT_CANCEL_HOME);
    const { user } = await open();
    const list = await screen.findByRole('group', {
      name: named(message(language, 'fruit-list-label')),
    });
    const row = await waitFor(() => {
      const found = list.querySelector<HTMLElement>('.ag-row[row-id="1"] .ag-cell');
      if (found === null) throw new Error('no row');
      return found;
    });
    await user.click(row);
    await user.click(screen.getByRole('button', { name: startsWith('fruit-cancel') }));
    await screen.findByRole('alertdialog');
    await user.type(field('fruit-cancel-reason'), 'not taken');
    await user.click(
      within(screen.getByRole('alertdialog')).getByRole('button', {
        name: startsWith('fruit-cancel-confirm'),
      }),
    );
    await waitFor(() => {
      expect(fruitApi.fruitCancel).toHaveBeenCalledTimes(1);
    });
    const sent = vi.mocked(fruitApi.fruitCancel).mock.calls[0]?.[0];
    if (sent === undefined) throw new Error('nothing sent');
    expect(sendsTheSame(sent, real.FRUIT_CANCEL_HOME_REQUEST)).toBeNull();
    await screen.findByText(
      said(message(language, 'fruit-cancelled-toast', { number: fruitNumber(1) })),
    );
    // Again: the backend refuses, and the words show inside the box with what was typed kept.
    vi.mocked(fruitApi.fruitCancel).mockRejectedValue(new CommandError(real.FRUIT_CANCEL_AGAIN));
    await user.click(screen.getByRole('button', { name: startsWith('fruit-cancel') }));
    const dialog = await screen.findByRole('alertdialog');
    await user.type(field('fruit-cancel-reason'), 'again');
    await user.click(
      within(dialog).getByRole('button', { name: startsWith('fruit-cancel-confirm') }),
    );
    await waitFor(() => {
      expect(visible(dialog.textContent)).toContain(message(language, 'err-fruit-cancelled'));
    });
  });
});
