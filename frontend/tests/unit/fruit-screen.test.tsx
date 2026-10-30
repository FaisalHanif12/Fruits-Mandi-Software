// The پھل screen (plan 17d, PRD 16.7, FR-16.10 to FR-16.12, FR-7.53, C27, E67), in the Urdu and the
// English app, against a fake backend with the numbers of E2E-42 (after E2E-16, 5 پیٹی کیلا are left
// in رحیم's lot): the tab next to خرچہ, the form with its two sources, the rate that is never filled
// from the hint, each partner's part before the save (one partner, or the guests shared equally,
// C27), the lot's balance, the refusals at their fields, the save, the list with its totals, the
// edit with its reason and the cancel with its reason, the keyboard-only run (F30) and a language
// switch in the middle of an entry that loses nothing (F39). Every figure on the screen is a figure
// the backend sent; the fake's sums are written the way the plan has them.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as fruitApi from '../../src/features/fruit/api';
import { LANGUAGES } from '../../src/i18n';
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
import { fakeFruit, fakeFruitPickers } from '../helpers/fruit';
import { EMPTY_PAIRS } from '../helpers/paged';
import { fakePartners, THREE_PARTNERS } from '../helpers/partners';
import { settlePreview } from '../helpers/preview-settled';
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

const HAJI = account(31, 'seller', '0012', 'حاجی نذیر', 'Haji Nazir');

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners(THREE_PARTNERS);
  fakeFruitPickers();
  fakeAccounts([HAJI]);
  fakeAccountSearch([HAJI]);
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

const said = (text: string) => (content: string) => visible(content) === text;
type User = ReturnType<typeof userEvent.setup>;

describe.each(LANGUAGES)('the fruit screen (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const partnerA = language === 'ur' ? 'الف' : 'Partner A';
  const rahim = language === 'ur' ? 'رحیم' : 'Rahim';
  const pageText = () => visible(document.body.textContent);
  const startsWith = (id: string) => (name: string) =>
    visible(name).startsWith(message(language, id));

  const open = async () => {
    const view = await renderApp('/rokar/fruit');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'fruit-title')),
    });
    return { ...view, user: userEvent.setup() };
  };
  /** A text field by its label: a plain box, or a picker or drop-down (a combobox). */
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
  /** Takes the lot of رحیم from the lot picker. */
  const pickLot = async (user: User) => {
    const box = field('fruit-field-lot');
    await user.click(box);
    await user.type(box, 'rahim');
    await user.click(await screen.findByRole('option'));
    await waitFor(() => {
      expect(box).toHaveValue(language === 'ur' ? 'کیلا پیٹی' : 'Banana Box');
    });
  };
  /** Waits until the last preview was asked for what is typed and has been answered. */
  async function previewSettled(rate: string) {
    await waitFor(() => {
      expect(fruitApi.fruitPreview).toHaveBeenLastCalledWith(expect.objectContaining({ rate }));
    });
    await act(async () => {
      await vi.mocked(fruitApi.fruitPreview).mock.results.at(-1)?.value;
    });
    await settlePreview();
  }
  const fruitNumber = (number: number) => message(language, 'fruit-number', { number });

  it('is the tab next to خرچہ: the strip names both and the خرچہ tab goes back', async () => {
    fakeFruit();
    const { user } = await open();
    const strip = screen.getByRole('navigation', {
      name: named(message(language, 'cashbook-tabs-label')),
    });
    const fruitTab = within(strip).getByRole('button', {
      name: named(message(language, 'cashbook-tab-fruit')),
    });
    expect(fruitTab).toHaveAttribute('aria-current', 'page');
    await user.click(
      within(strip).getByRole('button', { name: named(message(language, 'cashbook-tab-expense')) }),
    );
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'cashbook-expense-title')),
    });
  });

  it('shows the four states: a form with nothing typed, an empty list with its words, and no figures yet', async () => {
    fakeFruit();
    await open();
    expect(pageText()).toContain(message(language, 'fruit-numbers-empty'));
    expect(
      await screen.findByText(said(message(language, 'fruit-list-empty'))),
    ).toBeInTheDocument();
    expect(pageText()).toContain(message(language, 'fruit-list-empty-help'));
  });

  it('home fruit from our shop (E2E-42): the part comes off A, the lot goes from 5 to 3, and the rate is never filled', async () => {
    const fake = fakeFruit();
    const { user } = await open();
    await choose(user, 'fruit-field-for', partnerA);
    await pickLot(user);
    // The last rate is grey text only: the field stays empty (PRD FR-7.53).
    const rate = field('fruit-field-rate');
    expect(rate).toHaveValue('');
    await waitFor(() => {
      expect(rate).toHaveAttribute('placeholder', '580');
    });
    expect(pageText()).toContain(message(language, 'bikri-rate-hint-today', { rate: '580' }));
    expect(rate).toHaveValue('');
    await user.type(field('fruit-field-qty'), '2');
    await user.type(rate, '580');
    await previewSettled('580');
    // The backend's figures: value 1,160; A's total 1,424 becomes 264; the lot 5 becomes 3.
    await waitFor(() => {
      const text = pageText();
      for (const figure of ['1,160', '1,424', '264']) expect(text, figure).toContain(figure);
    });
    expect(pageText()).toContain(rahim);
    press('F10');
    await waitFor(() => {
      expect(fake.saved.entries).toHaveLength(1);
    });
    expect(fake.saved.entries[0]).toMatchObject({
      for_guests: false,
      partner_id: 1,
      source: 'shop',
      lot_id: 40,
      agent_account_id: null,
      item_id: null,
      packing_id: null,
      qty: '2',
      rate: '580',
    });
    // The toast names the new number; the list shows the entry with the backend's value.
    await screen.findByText(said(message(language, 'fruit-saved', { number: fruitNumber(1) })));
    await waitFor(() => {
      expect(pageText()).toContain('1,160');
    });
    expect(fake.lots[0]?.balance).toBe(3);
    // The next entry keeps the partner and the source, and the rest is empty.
    expect(field('fruit-field-qty')).toHaveValue('');
  });

  it('guest fruit is shared equally, 1,000 over 3 partners as 334 / 333 / 333 (C27)', async () => {
    fakeFruit();
    const { user } = await open();
    await choose(user, 'fruit-field-for', message(language, 'fruit-for-guests'));
    await pickLot(user);
    await user.type(field('fruit-field-qty'), '1');
    await user.type(field('fruit-field-rate'), '1000');
    await previewSettled('1000');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'fruit-parts-title-guests'));
    });
    const table = screen.getAllByRole('table')[0];
    if (table === undefined) throw new Error('no parts table');
    const rows = within(table).getAllByRole('row').slice(1);
    expect(rows).toHaveLength(3);
    // The partner's name is the first cell of the shared table; his part is the second (plan 39).
    expect(rows.map((row) => visible(within(row).getAllByRole('cell')[1]?.textContent))).toEqual([
      '334',
      '333',
      '333',
    ]);
    expect(pageText()).toContain(message(language, 'fruit-parts-note'));
  });

  it('fruit from another shop: the lot goes, the agent, item and packing come, and the save sends them', async () => {
    const fake = fakeFruit();
    const { user } = await open();
    await choose(user, 'fruit-field-for', message(language, 'fruit-for-guests'));
    await user.type(field('fruit-field-qty'), '3');
    await user.type(field('fruit-field-rate'), '1000');
    await user.click(
      within(
        screen.getByRole('group', { name: named(message(language, 'fruit-field-source')) }),
      ).getByRole('button', { name: named(message(language, 'fruit-source-other')) }),
    );
    // The lot field is gone and what was typed for the goods stays.
    expect(
      screen.queryByRole('combobox', { name: named(message(language, 'fruit-field-lot')) }),
    ).toBeNull();
    expect(field('fruit-field-qty')).toHaveValue('3');
    const agent = field('fruit-field-agent');
    await user.click(agent);
    await user.type(agent, 'nazir');
    await user.click(await screen.findByRole('option'));
    await waitFor(() => {
      expect(agent).not.toHaveValue('nazir');
    });
    const item = field('glossary-qism-maal');
    await user.click(item);
    await user.type(item, 'app');
    await user.click(
      await screen.findByRole('option', {
        name: (text) => visible(text).includes(language === 'ur' ? 'سیب' : 'Apple'),
      }),
    );
    await waitFor(() => {
      expect(item).not.toHaveValue('app');
    });
    const packing = field('glossary-packing');
    await user.click(packing);
    await user.type(packing, 'box');
    await user.click(await screen.findByRole('option'));
    await waitFor(() => {
      expect(packing).not.toHaveValue('box');
    });
    await previewSettled('1000');
    press('F10');
    await waitFor(() => {
      expect(fake.saved.entries).toHaveLength(1);
    });
    expect(fake.saved.entries[0]).toMatchObject({
      for_guests: true,
      partner_id: null,
      source: 'other_shop',
      lot_id: null,
      agent_account_id: 31,
      item_id: 63,
      packing_id: 71,
      qty: '3',
      rate: '1000',
    });
  });

  it('more than the lot has is refused at the quantity, with what is left; a rate of 0 at the rate', async () => {
    fakeFruit();
    const { user } = await open();
    await choose(user, 'fruit-field-for', partnerA);
    await pickLot(user);
    await user.type(field('fruit-field-qty'), '9');
    await user.type(field('fruit-field-rate'), '0');
    await previewSettled('0');
    await waitFor(() => {
      const text = pageText();
      expect(text).toContain(message(language, 'err-qty-exceeds-lot', { available: '5' }));
      expect(text).toContain(message(language, 'err-must-be-more-than-zero'));
    });
  });

  it('what is only empty is told when the user tries to save, at its field, and nothing is saved', async () => {
    const fake = fakeFruit();
    const { user } = await open();
    await pickLot(user);
    press('F10');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'err-partner-required'));
    });
    expect(fake.saved.entries).toHaveLength(0);
  });

  it('edits an entry: the form opens with its values, asks why, and sends the id, the version and the reason', async () => {
    const fake = fakeFruit();
    const { user } = await open();
    await choose(user, 'fruit-field-for', partnerA);
    await pickLot(user);
    await user.type(field('fruit-field-qty'), '2');
    await user.type(field('fruit-field-rate'), '580');
    await previewSettled('580');
    press('F10');
    await waitFor(() => {
      expect(fake.rows).toHaveLength(1);
    });
    // Select the row and edit it.
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
    expect(field('fruit-field-qty')).toHaveValue('2');
    expect(field('fruit-field-rate')).toHaveValue('580');
    // Save without a reason is refused at the reason field.
    await user.clear(field('fruit-field-qty'));
    await user.type(field('fruit-field-qty'), '1');
    await previewSettled('580');
    press('F10');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'err-reason-required'));
    });
    expect(fake.saved.edits).toHaveLength(0);
    await user.type(field('fruit-field-reason'), 'wrong count');
    press('F10');
    await waitFor(() => {
      expect(fake.saved.edits).toHaveLength(1);
    });
    expect(fake.saved.edits[0]).toMatchObject({ id: 1, version: 1, reason: 'wrong count' });
    expect(fake.saved.edits[0]?.entry).toMatchObject({ qty: '1', lot_id: 40 });
  });

  it('cancels an entry with a reason: the lot gets its fruit back and the entry stays in the list as cancelled', async () => {
    const fake = fakeFruit();
    const { user } = await open();
    await choose(user, 'fruit-field-for', partnerA);
    await pickLot(user);
    await user.type(field('fruit-field-qty'), '2');
    await user.type(field('fruit-field-rate'), '580');
    await previewSettled('580');
    press('F10');
    await waitFor(() => {
      expect(fake.rows).toHaveLength(1);
    });
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
    const dialog = await screen.findByRole('alertdialog');
    await user.type(field('fruit-cancel-reason'), 'mistake');
    await user.click(
      within(dialog).getByRole('button', { name: startsWith('fruit-cancel-confirm') }),
    );
    await waitFor(() => {
      expect(fake.saved.cancels).toHaveLength(1);
    });
    expect(fake.saved.cancels[0]).toMatchObject({ id: 1, version: 1, reason: 'mistake' });
    expect(fake.lots[0]?.balance).toBe(5);
    await screen.findByText(
      said(message(language, 'fruit-cancelled-toast', { number: fruitNumber(1) })),
    );
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'fruit-for-cancelled', { name: partnerA }));
    });
  });

  it('the keyboard does the rest: the lot, the quantity and the rate with Enter, then F10 (F30)', async () => {
    const fake = fakeFruit();
    const { user } = await open();
    // The drop-down is the shared Select (its own keys are tested with it); the rest is keys only.
    await choose(user, 'fruit-field-for', partnerA);
    // The lot: type, Enter takes the first match and the cursor moves on to the quantity.
    field('fruit-field-lot').focus();
    await user.keyboard('rahim');
    await screen.findByRole('option');
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(field('fruit-field-qty')).toHaveFocus();
    });
    await user.keyboard('2{Enter}');
    await waitFor(() => {
      expect(field('fruit-field-rate')).toHaveFocus();
    });
    await user.keyboard('580');
    await previewSettled('580');
    press('F10');
    await waitFor(() => {
      expect(fake.saved.entries).toHaveLength(1);
    });
    expect(fake.saved.entries[0]).toMatchObject({
      partner_id: 1,
      lot_id: 40,
      qty: '2',
      rate: '580',
    });
  });

  it('a partner the backend does not know is told at the partner field in plain words', async () => {
    const fake = fakeFruit();
    fake.refuse.save = [{ code: 'ERR_NOT_FOUND', field: 'partner_id' }];
    const { user } = await open();
    await choose(user, 'fruit-field-for', partnerA);
    await pickLot(user);
    await user.type(field('fruit-field-qty'), '2');
    await user.type(field('fruit-field-rate'), '580');
    await previewSettled('580');
    press('F10');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'err-not-found'));
    });
    expect(fake.saved.entries).toHaveLength(0);
    expect(field('fruit-field-qty')).toHaveValue('2');
  });

  it('a list that cannot be opened says so with a retry that asks again', async () => {
    fakeFruit();
    vi.mocked(fruitApi.fruitList).mockRejectedValueOnce(new Error('offline'));
    const { user } = await open();
    expect(
      await screen.findByText(said(message(language, 'fruit-list-failed'))),
    ).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: message(language, 'common-retry') }));
    expect(
      await screen.findByText(said(message(language, 'fruit-list-empty'))),
    ).toBeInTheDocument();
  });

  it('figures that cannot be worked out say so with a retry, and nothing typed is lost', async () => {
    fakeFruit();
    const { user } = await open();
    await choose(user, 'fruit-field-for', partnerA);
    vi.mocked(fruitApi.fruitPreview).mockRejectedValue(new Error('offline'));
    await pickLot(user);
    await user.type(field('fruit-field-qty'), '2');
    expect(
      await screen.findByText(said(message(language, 'fruit-numbers-failed'))),
    ).toBeInTheDocument();
    expect(field('fruit-field-qty')).toHaveValue('2');
    const retry = screen.getAllByRole('button', { name: message(language, 'common-retry') });
    expect(retry.length).toBeGreaterThan(0);
  });

  it('a language switch in the middle of an entry loses nothing', async () => {
    fakeFruit();
    const { user } = await open();
    await choose(user, 'fruit-field-for', partnerA);
    await pickLot(user);
    await user.type(field('fruit-field-qty'), '2');
    await user.type(field('fruit-field-rate'), '580');
    await previewSettled('580');
    press('KeyL', { altKey: true });
    const other = language === 'ur' ? 'en' : 'ur';
    await screen.findByRole('heading', { level: 1, name: named(message(other, 'fruit-title')) });
    const byLabel = (id: string) => {
      const label = named(message(other, id));
      return (
        screen.queryByRole('textbox', { name: label }) ??
        screen.getByRole('combobox', { name: label })
      );
    };
    expect(byLabel('fruit-field-qty')).toHaveValue('2');
    expect(byLabel('fruit-field-rate')).toHaveValue('580');
    expect(byLabel('fruit-field-lot')).toHaveValue(other === 'ur' ? 'کیلا پیٹی' : 'Banana Box');
    expect(byLabel('fruit-field-for')).toHaveTextContent(other === 'ur' ? 'الف' : 'Partner A');
  });
});
