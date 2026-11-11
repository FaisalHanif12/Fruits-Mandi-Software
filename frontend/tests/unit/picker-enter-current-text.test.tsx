// D8 Bug 1, real cause (plan 61, PRD FR-7.14, FR-7.16, FR-7.39): Umar's third line held buyer 0030,
// the first row of the list that opens with nothing typed, and no text. He had typed ہارون. Enter
// must never pick from a list that does not belong to the text now typed: it waits for the answer
// to that text and picks its first row; if the text matches nothing it stays in the cell and says
// so; a short code still resolves. The real picker cell of the real sale grid, the real search
// hook, and a backend that answers a typed text late (as the real one does under load), with the
// hits of Umar's own database: the list with nothing typed starts with 0030, then انس, مانی,
// نقدی منڈی and ہارون; every text of ہارون finds only ہارون.

import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as accountApi from '../../src/shared/accounts/api';
import * as urduApi from '../../src/shared/urdu/api';
import { account, fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeBikri } from '../helpers/bikri';
import { EHSAN, umarAnswer } from '../helpers/bikri-umar-data';
import { seedUmarsLot } from '../helpers/bikri-umar';
import { cellOf, combo, focusedIn, salesGrid, textOf } from '../helpers/bikri-screen';
import { fakePartners } from '../helpers/partners';
import { EMPTY_PAIRS } from '../helpers/paged';
import { message } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

/** A wait that holds when the whole suite is running and the machine is busy. */
const SLOW_MS = { timeout: 8_000 };
/** How late the backend answers a typed text: long enough for Enter to come first. */
const ANSWER_MS = 600;

// The buyers of Umar's database, with the codes and the order the real search gave.
const NAQDI = account(8, 'buyer', '03040000', 'نقدی منڈی', 'Naqdi Mandi');
const MANI = account(26, 'buyer', '0020', 'مانی', 'Mani');
const HAROON = account(27, 'buyer', '0021', 'ہارون', 'Haroon');
const ANAS = account(28, 'buyer', '0022', 'انس', 'Anas');
const CHUKTA = account(9, 'buyer', '0030', 'چکتا کھاتہ', 'Chukta Khata');
/** The list with nothing typed: recent first, as the backend gave it to Umar. */
const EMPTY_TEXT_ORDER = [CHUKTA, ANAS, MANI, NAQDI, HAROON];

const wait = (ms: number) =>
  new Promise<void>((resolve) => {
    setTimeout(resolve, ms);
  });

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
});

afterEach(() => {
  leaveAppWindow();
});

/**
 * Opens the sale book with Umar's lot and an empty first line, with the search of the buyer field
 * the way Umar's backend answered it: the empty text at once, in the order above, and every typed
 * text only after `ANSWER_MS`.
 */
async function openWithSlowSearch(language: Language) {
  const rows = [EHSAN, ...EMPTY_TEXT_ORDER];
  fakeBikri(umarAnswer);
  fakeAccounts(rows);
  fakeAccountSearch(rows);
  const quick = vi.mocked(accountApi.searchAccounts).getMockImplementation();
  if (quick === undefined) throw new Error('the fake search is not installed');
  const asked: string[] = [];
  vi.mocked(accountApi.searchAccounts).mockImplementation(async (input) => {
    if (input.scope === 'buyer') asked.push(input.text);
    if (input.scope === 'buyer' && input.text === '') {
      const page = EMPTY_TEXT_ORDER.slice(input.offset ?? 0, (input.offset ?? 0) + input.limit);
      return (await quick({ ...input, text: '' })).sort(
        (a, b) =>
          page.findIndex((row) => row.id === a.id) - page.findIndex((row) => row.id === b.id),
      );
    }
    await wait(ANSWER_MS);
    return quick(input);
  });
  seedUmarsLot();
  const app = await renderApp('/bikri/book');
  await screen.findByRole('heading', { level: 1, name: message(language, 'glossary-bikri-book') });
  const user = userEvent.setup();
  const sales = salesGrid(language);
  await waitFor(() => {
    expect(cellOf(sales, 0, 'buyer')).toBeInTheDocument();
  }, SLOW_MS);
  await user.click(cellOf(sales, 0, 'buyer'));
  await waitFor(() => {
    expect(focusedIn(sales)).toBe('0:buyer');
  }, SLOW_MS);
  // The list that opens with nothing typed: 0030 is its first row.
  await waitFor(() => {
    expect(screen.queryAllByRole('option').length).toBe(EMPTY_TEXT_ORDER.length);
  }, SLOW_MS);
  return { ...app, user, sales, asked };
}

const NAME = (language: Language, row: typeof HAROON) =>
  language === 'ur' ? row.name_ur : row.name_en;

describe.each(LANGUAGES)('Enter in a buyer cell of the sale lines (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it.each(['ہارون', 'Haroon', 'haroon', 'ہارو'])(
    'typing %s and pressing Enter before the answer lands picks ہارون, not the first row of the empty list',
    async (text) => {
      const { user, sales } = await openWithSlowSearch(language);
      // The list shown now is the one for nothing typed, with 0030 on top. It is not an answer to
      // the text typed next, so Enter must not pick from it.
      expect(screen.getAllByRole('option')[0]?.textContent).toContain('0030');
      await user.keyboard(`${text}{Enter}`);
      await waitFor(() => {
        expect(focusedIn(sales)).toBe('0:qty');
      }, SLOW_MS);
      expect(textOf(sales, 0, 'buyer')).toBe(NAME(language, HAROON));
    },
  );

  it('typing ہارون and waiting for its list, then Enter, picks ہارون', async () => {
    const { user, sales } = await openWithSlowSearch(language);
    await user.keyboard('ہارون');
    await waitFor(() => {
      expect(screen.getAllByRole('option')).toHaveLength(1);
    }, SLOW_MS);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:qty');
    }, SLOW_MS);
    expect(textOf(sales, 0, 'buyer')).toBe(NAME(language, HAROON));
  });

  it('a short code typed and accepted with Enter before the answer still resolves to its buyer', async () => {
    const { user, sales } = await openWithSlowSearch(language);
    await user.keyboard('1{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:qty');
    }, SLOW_MS);
    expect(textOf(sales, 0, 'buyer')).toBe(NAME(language, HAROON));
  });

  it('Enter on a text that matches nothing stays in the cell, picks nothing and goes nowhere', async () => {
    const { user, sales } = await openWithSlowSearch(language);
    await user.keyboard('zzzz{Enter}');
    // The answer is empty, so nothing is picked and the cell is not left (the add form of FR-7.69
    // opens with the text, which is how the picker says "no such buyer").
    await waitFor(() => {
      expect(screen.queryByRole('dialog') !== null || screen.queryByRole('status') !== null).toBe(
        true,
      );
    }, SLOW_MS);
    expect(textOf(sales, 0, 'buyer')).toBe('');
    expect(focusedIn(sales)).not.toBe('0:qty');
  });

  it('Enter on an empty cell, with the empty list open and nothing pointed at, picks nothing', async () => {
    const { user, sales } = await openWithSlowSearch(language);
    await act(async () => {
      await wait(50);
    });
    await user.keyboard('{Enter}');
    expect(textOf(sales, 0, 'buyer')).toBe('');
  });

  it('typing a letter, deleting it and pressing Enter picks nothing: the empty list has no highlight', async () => {
    const { user, sales } = await openWithSlowSearch(language);
    await user.keyboard('ہ{Backspace}');
    await user.keyboard('{Enter}');
    // The cell is empty, so Enter keeps it empty and moves on (FR-7.14); the first row of the full
    // list, 0030, is never taken for him.
    expect(textOf(sales, 0, 'buyer')).toBe('');
    expect(screen.queryByText(/0030/)).not.toBeInTheDocument();
  });
});

// The same rule in the pickers of a form (the زمیندار of the header): the list closed with Esc
// still leaves the text in the field, and Enter on it must resolve that text, not leave it behind.
describe.each(LANGUAGES)('Enter in the زمیندار picker of the header (%s)', (language) => {
  const KARIM = account(30, 'seller', '0011', 'کریم', 'Karim');
  const SELLERS = [KARIM, EHSAN];

  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  async function openHeader() {
    fakeBikri(umarAnswer);
    fakeAccounts([...SELLERS, ...EMPTY_TEXT_ORDER]);
    fakeAccountSearch([...SELLERS, ...EMPTY_TEXT_ORDER]);
    const quick = vi.mocked(accountApi.searchAccounts).getMockImplementation();
    if (quick === undefined) throw new Error('the fake search is not installed');
    vi.mocked(accountApi.searchAccounts).mockImplementation(async (input) => {
      if (input.text === '') {
        const hits = await quick(input);
        return [...hits].sort((a, b) => b.id - a.id);
      }
      await wait(ANSWER_MS);
      return quick(input);
    });
    await renderApp('/bikri/book');
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, 'glossary-bikri-book'),
    });
    const user = userEvent.setup();
    const zamindar = await combo(language, 'glossary-zamindar');
    await user.click(zamindar);
    await waitFor(() => {
      expect(screen.queryAllByRole('option').length).toBe(SELLERS.length);
    }, SLOW_MS);
    return { user, zamindar };
  }

  const EHSAN_NAME = language === 'ur' ? 'احسان' : 'Ehsan';

  it('text typed, the list closed with Esc, then Enter: the text is answered and picked, not dropped', async () => {
    const { user, zamindar } = await openHeader();
    await user.keyboard('ehsan{Escape}');
    expect(screen.queryAllByRole('option')).toHaveLength(0);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(zamindar).toHaveValue(EHSAN_NAME);
    }, SLOW_MS);
  });

  it('text typed and Enter before the answer lands picks the match of that text, not the first row of the full list', async () => {
    const { user, zamindar } = await openHeader();
    await user.keyboard('ehsan{Enter}');
    await waitFor(() => {
      expect(zamindar).toHaveValue(EHSAN_NAME);
    }, SLOW_MS);
  });

  it('a letter typed and deleted, then Enter, picks nothing', async () => {
    const { user, zamindar } = await openHeader();
    await user.keyboard('e{Backspace}{Enter}');
    expect(zamindar).toHaveValue('');
  });
});
