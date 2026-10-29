// Goods arrived on the garden screen (plan 33 version 1.1, PRD FR-12.40, FR-7.67), in the Urdu and
// the English app. The type list has a fourth type, مال آیا; its form has the item, the packing, the
// quantity and the day to sell, and the screen says that it posts no money. The list of the garden's
// entries shows what is left and the day to sell; the card has the actions: change the day to sell,
// write off, undo a write-off, cancel. The answers and the requests are the backend's real files
// (arrivals_fixtures.rs).

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as gardensApi from '../../src/features/gardens/api';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import * as arrivals from '../helpers/arrivals-real';
import { named } from '../helpers/bikri-screen';
import {
  fail,
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeGardens } from '../helpers/gardens';
import { EMPTY_PAIRS } from '../helpers/paged';
import { fakePartners, THREE_PARTNERS } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { fakeItemPickers } from '../helpers/trips';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/gardens/api');
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
  fakeItemPickers();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

const said = (text: string) => (content: string) => visible(content) === text;
type User = ReturnType<typeof userEvent.setup>;

describe.each(LANGUAGES)('goods arrived on the garden screen (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const open = async () => {
    const view = await renderApp('/gardens/1');
    await screen.findByText(said(message(language, 'gardens-detail-title', { code: '0080' })));
    return { ...view, user: userEvent.setup() };
  };

  const typeList = () =>
    screen.getByRole('group', { name: named(message(language, 'gardens-type-label')) });
  const chooseArrival = async (user: User) => {
    await user.click(
      within(typeList()).getByRole('button', {
        name: named(message(language, 'gardens-type-arrival')),
      }),
    );
  };
  const field = (id: string) => {
    const label = named(message(language, id));
    return (
      screen.queryByRole('textbox', { name: label }) ??
      screen.getByRole('combobox', { name: label })
    );
  };
  async function pick(user: User, id: string, text: string) {
    const box = field(id);
    await user.click(box);
    await user.type(box, text);
    await user.click(await screen.findByRole('option'));
  }
  const dayText = (box: HTMLElement) => (box as HTMLInputElement).value;

  it('the type list has the goods arrived; its form has the item, the packing, the quantity and the day to sell, and the screen says it posts no money', async () => {
    fakeGardens();
    const { user } = await open();
    expect(
      within(typeList())
        .getAllByRole('button')
        .map((button) => visible(button.textContent)),
    ).toContain(message(language, 'gardens-type-arrival'));
    await chooseArrival(user);
    expect(field('glossary-qism-maal')).toBeInTheDocument();
    expect(field('glossary-packing')).toBeInTheDocument();
    expect(field('gardens-arrival-qty')).toBeInTheDocument();
    // The day to sell starts as the day of the arrival, visible like a typed value (NFR-USE-27).
    const sell = field('glossary-sell-date');
    const day = field('gardens-field-date');
    expect(dayText(sell)).toBe(dayText(day));
    expect(
      screen.getByText(said(message(language, 'gardens-arrival-posts-nothing'))),
    ).toBeInTheDocument();
    // No sources of money: nothing is paid.
    expect(screen.queryByText(said(message(language, 'gardens-sources-title')))).toBeNull();
    // It follows the day of the arrival while nobody has typed in it.
    await user.click(day);
    await user.keyboard('{Control>}a{/Control}09-10-2026');
    await waitFor(() => {
      expect(dayText(sell)).toBe('09-10-2026');
    });
  });

  it('F10 saves the goods arrived as typed, with the day to sell the user typed, and starts the next one', async () => {
    fakeGardens();
    vi.mocked(gardensApi.gardensArrivalSave).mockResolvedValue(arrivals.ARRIVAL_SAVE);
    const { user } = await open();
    await chooseArrival(user);
    await pick(user, 'glossary-qism-maal', language === 'ur' ? 'م' : 'Ora');
    await pick(user, 'glossary-packing', language === 'ur' ? 'پ' : 'Box');
    await user.type(field('gardens-arrival-qty'), '100');
    const sell = field('glossary-sell-date');
    await user.click(sell);
    await user.keyboard('{Control>}a{/Control}12-10-2026');
    press('F10');
    await waitFor(() => {
      expect(gardensApi.gardensArrivalSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(gardensApi.gardensArrivalSave).mock.calls[0]?.[0]).toMatchObject({
      garden_id: 1,
      sell_date: '12-10-2026',
      qty: '100',
      weight: null,
      note: null,
    });
    expect(
      await screen.findByText(
        said(
          message(language, 'gardens-saved-arrival', {
            number: message(language, 'gardens-arrival-number', { number: 1 }),
          }),
        ),
      ),
    ).toBeInTheDocument();
    // The next entry starts empty and its day to sell is the day again.
    await waitFor(() => {
      expect(field('gardens-arrival-qty')).toHaveValue('');
    });
  });

  it('an untouched day to sell is sent as nothing: the backend reads it as the day of the arrival', async () => {
    fakeGardens();
    vi.mocked(gardensApi.gardensArrivalSave).mockResolvedValue(arrivals.ARRIVAL_SAVE);
    const { user } = await open();
    await chooseArrival(user);
    await pick(user, 'glossary-qism-maal', language === 'ur' ? 'م' : 'Ora');
    await pick(user, 'glossary-packing', language === 'ur' ? 'پ' : 'Box');
    await user.type(field('gardens-arrival-qty'), '100');
    press('F10');
    await waitFor(() => {
      expect(gardensApi.gardensArrivalSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(gardensApi.gardensArrivalSave).mock.calls[0]?.[0].sell_date).toBeNull();
  });

  it('a day to sell before the day of the arrival is refused at the field, and what was typed stays', async () => {
    fakeGardens();
    vi.mocked(gardensApi.gardensArrivalSave).mockRejectedValue(
      fail({
        code: 'ERR_SELL_DATE_BEFORE_ENTRY',
        field: 'sell_date',
        values: { entry_date: '2026-10-11' },
      }),
    );
    const { user } = await open();
    await chooseArrival(user);
    await pick(user, 'glossary-qism-maal', language === 'ur' ? 'م' : 'Ora');
    await pick(user, 'glossary-packing', language === 'ur' ? 'پ' : 'Box');
    await user.type(field('gardens-arrival-qty'), '100');
    await user.click(field('glossary-sell-date'));
    await user.keyboard('{Control>}a{/Control}01-01-2020');
    press('F10');
    expect(
      await screen.findByText(
        said(message(language, 'err-sell-date-before-entry', { entry_date: '11-10-2026' })),
      ),
    ).toBeInTheDocument();
    expect(field('gardens-arrival-qty')).toHaveValue('100');
    expect(dayText(field('glossary-sell-date'))).toBe('01-01-2020');
  });

  it('lists the entries of the garden with what is left, and the card has the figures and the day to sell', async () => {
    fakeGardens();
    vi.mocked(gardensApi.gardensArrivalsList).mockResolvedValue(arrivals.ARRIVALS_IN_STOCK);
    await open();
    const row = await waitFor(() => {
      const found = document.querySelector('.ag-row[row-id="1"]');
      if (found === null) throw new Error('no row');
      return found;
    });
    expect(visible(row.querySelector('[col-id="number"]')?.textContent)).toBe(
      message(language, 'gardens-arrival-number', { number: 1 }),
    );
    expect(visible(row.querySelector('[col-id="qty_left"]')?.textContent)).toBe('100');
    const card = (
      await screen.findByText(
        said(
          message(language, 'gardens-arrival-card-title', {
            number: message(language, 'gardens-arrival-number', { number: 1 }),
            item: language === 'ur' ? 'کیلا' : 'Banana',
          }),
        ),
      )
    ).parentElement;
    const text = visible(card?.textContent);
    for (const figure of ['11-10-2026', '12-10-2026', '100']) expect(text).toContain(figure);
    // The garden's list was asked for this garden, the first page.
    expect(gardensApi.gardensArrivalsList).toHaveBeenCalledWith(
      expect.objectContaining({ garden_id: 1, offset: 0 }),
    );
  });

  it('a garden with no goods arrived says what to do', async () => {
    fakeGardens();
    await open();
    expect(
      await screen.findByText(said(message(language, 'gardens-arrival-empty'))),
    ).toBeInTheDocument();
  });

  it('change the day to sell: the box starts with the day now, and the new day goes with the version', async () => {
    fakeGardens();
    vi.mocked(gardensApi.gardensArrivalsList).mockResolvedValue(arrivals.ARRIVALS_IN_STOCK);
    vi.mocked(gardensApi.gardensArrivalSetSellDate).mockResolvedValue(
      arrivals.ARRIVAL_SET_SELL_DATE,
    );
    const { user } = await open();
    await user.click(
      await screen.findByRole('button', {
        name: said(message(language, 'purchases-change-sell-date')),
      }),
    );
    const dialog = await screen.findByRole('dialog');
    const box = within(dialog).getByRole<HTMLInputElement>('textbox', {
      name: named(message(language, 'glossary-sell-date')),
    });
    expect(box.value).toBe('12-10-2026');
    await user.click(box);
    await user.keyboard('{Control>}a{/Control}13-10-2026');
    await user.click(
      within(dialog).getByRole('button', { name: said(message(language, 'common-save')) }),
    );
    await waitFor(() => {
      expect(gardensApi.gardensArrivalSetSellDate).toHaveBeenCalledWith({
        id: 1,
        version: 3,
        sell_date: '13-10-2026',
      });
    });
  });

  it('write off: the quantity and the reason go to the backend; more than is left is refused in the box', async () => {
    fakeGardens();
    vi.mocked(gardensApi.gardensArrivalsList).mockResolvedValue(arrivals.ARRIVALS_IN_STOCK);
    vi.mocked(gardensApi.gardensArrivalWriteOff).mockRejectedValueOnce(
      fail({ code: 'ERR_WRITE_OFF_EXCEEDS_BALANCE', field: 'qty', values: { available: 100 } }),
    );
    const { user } = await open();
    await user.click(
      await screen.findByRole('button', { name: said(message(language, 'purchases-write-off')) }),
    );
    const dialog = await screen.findByRole('alertdialog');
    await user.type(
      within(dialog).getByRole('textbox', {
        name: named(message(language, 'gardens-arrival-write-off-qty')),
      }),
      '500',
    );
    await user.type(
      within(dialog).getByRole('combobox', {
        name: named(message(language, 'gardens-arrival-reason')),
      }),
      'Spoiled',
    );
    await user.click(
      within(dialog).getByRole('button', {
        name: said(message(language, 'gardens-arrival-write-off-confirm')),
      }),
    );
    expect(
      await within(dialog).findByText(
        said(message(language, 'err-write-off-exceeds-balance', { available: 100 })),
      ),
    ).toBeInTheDocument();
    expect(gardensApi.gardensArrivalWriteOff).toHaveBeenCalledWith({
      arrival_id: 1,
      qty: '500',
      reason: 'Spoiled',
    });
  });

  it('cancel: the reason goes to the backend with the version; an entry with sales cannot be cancelled', async () => {
    fakeGardens();
    vi.mocked(gardensApi.gardensArrivalsList).mockResolvedValue(arrivals.ARRIVALS_IN_STOCK);
    vi.mocked(gardensApi.gardensArrivalCancel).mockResolvedValue(arrivals.ARRIVAL_CANCEL);
    const { user } = await open();
    await screen.findByText(
      said(
        message(language, 'gardens-arrival-card-title', {
          number: message(language, 'gardens-arrival-number', { number: 1 }),
          item: language === 'ur' ? 'کیلا' : 'Banana',
        }),
      ),
    );
    await user.click(
      await screen.findByRole('button', {
        name: said(message(language, 'gardens-arrival-cancel')),
      }),
    );
    const dialog = await screen.findByRole('alertdialog');
    await user.type(
      within(dialog).getByRole('combobox', {
        name: named(message(language, 'gardens-arrival-reason')),
      }),
      'Wrong',
    );
    await user.click(
      within(dialog).getByRole('button', {
        name: said(message(language, 'gardens-arrival-cancel-confirm')),
      }),
    );
    await waitFor(() => {
      expect(gardensApi.gardensArrivalCancel).toHaveBeenCalledWith({
        id: 1,
        version: 3,
        reason: 'Wrong',
      });
    });
  });

  it('an entry that was sold in part cannot be cancelled: the button is dimmed', async () => {
    fakeGardens();
    vi.mocked(gardensApi.gardensArrivalsList).mockResolvedValue({
      ...arrivals.ARRIVALS_IN_STOCK,
      rows: [arrivals.ARRIVAL_AFTER_A_SALE],
    });
    await open();
    expect(
      await screen.findByRole('button', {
        name: said(message(language, 'gardens-arrival-cancel')),
      }),
    ).toBeDisabled();
  });
});
