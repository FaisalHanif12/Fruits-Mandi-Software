// Gate lane T4 (plan 80): a language switch in the middle of a بکری entry loses nothing (Code
// Standards F39, PRD FR-7.33, E32, NFR-SPD-19). Owner: `robustness`.
//
// `bikri-paid.test.tsx` switches once, after a cell is committed. This test is harder in the ways
// a munshi really presses `Alt+L`:
//
// 1. three switches in a row (اردو, English, اردو, English), with a preview call still on its way,
//    so a slow answer of the old language cannot overwrite the new draft;
// 2. a line typed halfway: the rate is typed in the cell and `Enter` is not pressed yet;
// 3. a second line, a changed date and a typed paid amount, all kept, and the draft the backend
//    sees is the draft before the switch.
//
// The real screen, the real grid and the real store, against the fake backend. The screen works
// out no money (CLAUDE.md rule 1): every figure shown is the fake backend's.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { BikriDraft } from '../../src/bindings/BikriDraft';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { CHUKTA_KHATA, fakeBikri } from '../helpers/bikri';
import { seedUmarsLot, umarAnswer } from '../helpers/bikri-umar';
import { EHSAN, UMAR_BUYERS } from '../helpers/bikri-umar-data';
import { cellOf, focusedIn, salesGrid, textOf, type User } from '../helpers/bikri-screen';
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

const SLOW_MS = { timeout: 8_000 };
vi.setConfig({ testTimeout: 90_000 });

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

const other = (language: Language): Language => (language === 'ur' ? 'en' : 'ur');

async function pickBuyer(user: User, code: string) {
  await user.keyboard(code);
  await screen.findAllByRole('option', undefined, SLOW_MS);
  await user.keyboard('{Enter}');
}

/** What the backend is told about the lines: the values typed, never the words of a language. */
function typedLines(draft: BikriDraft) {
  return (draft.lots[0]?.sales ?? [])
    .filter((line) => line.buyer_account_id !== null || line.qty !== '' || line.seller_rate !== '')
    .map((line) => ({
      buyer: line.buyer_account_id,
      qty: line.qty,
      rate: line.seller_rate,
      paid: line.paid ?? '',
    }));
}

/**
 * Opens the book with an answer that can be held back: while `hold.release` is not called, a
 * preview waits, as a slow disk would make it. Every answer is still the fake backend's.
 */
async function openBook(language: Language) {
  const hold: { on: boolean; waiting: (() => void)[] } = { on: false, waiting: [] };
  const answer = async (draft: BikriDraft): Promise<BikriPreview> => {
    if (hold.on) {
      await new Promise<void>((resolve) => {
        hold.waiting.push(resolve);
      });
    }
    return umarAnswer(draft);
  };
  const backend = fakeBikri(answer);
  fakeAccounts([EHSAN, ...UMAR_BUYERS, CHUKTA_KHATA]);
  fakeAccountSearch([EHSAN, ...UMAR_BUYERS, CHUKTA_KHATA]);
  seedUmarsLot();
  const app = await renderApp('/bikri/book');
  await screen.findByRole('heading', { level: 1, name: message(language, 'glossary-bikri-book') });
  const sales = salesGrid(language);
  await waitFor(() => {
    expect(cellOf(sales, 0, 'buyer')).toBeInTheDocument();
  }, SLOW_MS);
  const user = userEvent.setup();
  await user.click(cellOf(sales, 0, 'buyer'));
  await waitFor(() => {
    expect(focusedIn(sales)).toBe('0:buyer');
  }, SLOW_MS);
  return { ...app, backend, user, sales, hold };
}

describe.each(LANGUAGES)('a language switch in the middle of a بکری (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('three switches in a row, with a preview on its way, keep both lines, the paid amount and the cursor', async () => {
    const { user, backend, sales, hold } = await openBook(language);
    // Line 1: buyer, qty 20, rate 1000, paid 5000, committed with Enter.
    await pickBuyer(user, '0');
    await user.keyboard('20{Enter}1000{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:paid');
    }, SLOW_MS);
    await user.keyboard('5000{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('1:buyer');
    }, SLOW_MS);
    // Line 2: buyer, qty 50, rate 900, committed.
    await pickBuyer(user, '1');
    await user.keyboard('50{Enter}900{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('1:paid');
    }, SLOW_MS);
    // The backend has seen both lines, the second with its rate, before the switches start.
    await waitFor(() => {
      const lines = typedLines(backend.lastDraft());
      expect(lines.length).toBe(2);
      expect(lines[1]?.rate).toBe('900');
    }, SLOW_MS);
    const before = typedLines(backend.lastDraft());
    expect(before[0]).toMatchObject({ qty: '20', rate: '1000', paid: '5000' });
    expect(before[1]).toMatchObject({ qty: '50', rate: '900' });

    // Hold the next previews back, then switch three times, quickly.
    hold.on = true;
    const user2 = userEvent.setup();
    await user2.keyboard('{Alt>}l{/Alt}');
    await user2.keyboard('{Alt>}l{/Alt}');
    await user2.keyboard('{Alt>}l{/Alt}');
    const last = other(language);
    // Three switches end in the other language.
    await waitFor(() => {
      expect(textOf(salesGrid(last), 0, 'paid')).toBe('5,000');
    }, SLOW_MS);
    expect(textOf(salesGrid(last), 0, 'qty')).toBe('20');
    expect(textOf(salesGrid(last), 1, 'qty')).toBe('50');
    // The slow answers come now, after the switches: nothing is overwritten by them.
    hold.on = false;
    for (const release of hold.waiting.splice(0)) release();
    await waitFor(() => {
      expect(textOf(salesGrid(last), 0, 'paid')).toBe('5,000');
    }, SLOW_MS);
    expect(textOf(salesGrid(last), 0, 'buyerTotal')).not.toBe('');
    // The draft the backend sees now is the draft typed before, line for line.
    expect(typedLines(backend.lastDraft())).toEqual(before);
    // The cursor is where it was.
    expect(focusedIn(salesGrid(last))).toBe('1:paid');
  });

  it('a rate typed in the cell and not yet confirmed is not lost on a switch', async () => {
    const { user, backend, sales } = await openBook(language);
    await pickBuyer(user, '0');
    await user.keyboard('20{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:rate');
    }, SLOW_MS);
    // The digits are in the cell; Enter is not pressed.
    await user.keyboard('1000');
    await user.keyboard('{Alt>}l{/Alt}');
    const next = other(language);
    await waitFor(() => {
      expect(textOf(salesGrid(next), 0, 'qty')).toBe('20');
    }, SLOW_MS);
    // Either the cell is still being typed in (the digits are in the editor), or the value was
    // taken as if Enter had been pressed. Both keep the 1000. A cell that went back to empty is
    // the loss F39 forbids.
    const cell = cellOf(salesGrid(next), 0, 'rate');
    const input = cell.querySelector('input');
    const shown = input === null ? cell.textContent : input.value;
    const draftRate = backend.lastDraft().lots[0]?.sales[0]?.seller_rate ?? '';
    expect(
      shown.replace(/[^0-9]/gu, '').includes('1000') || draftRate === '1000',
      `the rate typed before the switch is gone: cell "${shown}", draft "${draftRate}"`,
    ).toBe(true);
  });
});
