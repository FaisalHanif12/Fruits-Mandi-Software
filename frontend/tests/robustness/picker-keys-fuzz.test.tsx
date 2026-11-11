// D8 end gate, attack on the keyboard of the sale lines (D8 Bug 1 real cause, Bug 2; PRD FR-7.14, FR-7.16,
// FR-7.39, FR-7.91; NFR-USE-7). Owner: `robustness`.
//
// A seeded fuzz on the real sale book with the real grid and the real picker, against a backend that answers
// a typed text late (the real one does under load). Each trial types, deletes, clears and presses Enter in the
// buyer cell of line 1 in a random order, and presses Backspace and Delete in the other cells of the line.
// Invariants:
//
//   1. The leave box ("save changes?") never opens, and the sale book is still the screen (Backspace and
//      Delete only delete, FR-7.91).
//   2. Enter with nothing typed picks nothing: the empty list has no highlight, so its first row is never
//      taken (Umar's case: the first row was the one-time account).
//   3. Enter with text typed picks the first row of the ANSWER TO THAT TEXT, however late it lands, or
//      nothing when the text matches nothing. Never a row of another text.
//   4. A picked buyer is shown by his name in the cell. A cell that picked nothing never shows a record.
//
// It does not click the mouse or open the add form. The real timing on WebView2 is on the Windows list.

import { cleanup, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterAll, afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { Language } from '../../src/i18n';
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
import { cellOf, focusedIn, salesGrid, textOf } from '../helpers/bikri-screen';
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

const SLOW_MS = { timeout: 10_000 };

const NAQDI = account(8, 'buyer', '03040000', 'نقدی منڈی', 'Naqdi Mandi');
const MANI = account(26, 'buyer', '0020', 'مانی', 'Mani');
const HAROON = account(27, 'buyer', '0021', 'ہارون', 'Haroon');
const ANAS = account(28, 'buyer', '0022', 'انس', 'Anas');
const AKRAM = account(29, 'buyer', '0023', 'اکرم', 'Akram');
const BUYERS = [ANAS, MANI, NAQDI, HAROON, AKRAM];
/** The list with nothing typed: a decoy at the top, as the real one gave (recent first). */
const EMPTY_TEXT_ORDER = BUYERS;

const TEXTS = [
  'ہ',
  'ار',
  'ہارون',
  'Haroon',
  'haroon',
  'har',
  'ak',
  'اکرم',
  '1',
  '2',
  '0',
  '3',
  'zzzz',
  'ا',
  'م',
];

const wait = (ms: number) =>
  new Promise<void>((resolve) => {
    setTimeout(resolve, ms);
  });

/** A small seeded generator. */
function rng(seed: number) {
  let state = seed * 2_654_435_761 + 12_345;
  const next = () => {
    state = (state * 1_664_525 + 1_013_904_223) % 4_294_967_296;
    return state / 4_294_967_296;
  };
  return {
    below: (n: number) => Math.floor(next() * n),
    pick: <T,>(items: readonly T[]): T => items[Math.floor(next() * items.length)] as T,
  };
}

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
});

afterEach(() => {
  cleanup();
  leaveAppWindow();
});

async function openWithSlowSearch(language: Language, answerMs: number) {
  const rows = [EHSAN, ...EMPTY_TEXT_ORDER];
  const backend = fakeBikri(umarAnswer);
  fakeAccounts(rows);
  fakeAccountSearch(rows);
  const quick = vi.mocked(accountApi.searchAccounts).getMockImplementation();
  if (quick === undefined) throw new Error('the fake search is not installed');
  vi.mocked(accountApi.searchAccounts).mockImplementation(async (input) => {
    if (input.scope === 'buyer' && input.text === '') {
      const page = EMPTY_TEXT_ORDER.slice(input.offset ?? 0, (input.offset ?? 0) + input.limit);
      return (await quick({ ...input, text: '' })).sort(
        (a, b) =>
          page.findIndex((row) => row.id === a.id) - page.findIndex((row) => row.id === b.id),
      );
    }
    await wait(answerMs);
    return quick(input);
  });
  seedUmarsLot();
  await renderApp('/bikri/book');
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
  await waitFor(() => {
    expect(screen.queryAllByRole('option').length).toBe(EMPTY_TEXT_ORDER.length);
  }, SLOW_MS);
  return { user, sales, backend, quick: quick };
}

const NAME = (language: Language, row: typeof HAROON) =>
  language === 'ur' ? row.name_ur : row.name_en;

type Search = typeof accountApi.searchAccounts;

/** What the real search answers for a text: the model of rule 3. */
async function answerFor(quick: Search, text: string) {
  return quick({
    scope: 'buyer',
    control_id: null,
    text,
    limit: 50,
    offset: 0,
    with_chukta: true,
  });
}

function noLeaveBox(): void {
  expect(screen.queryByRole('alertdialog'), 'the leave box opened').toBeNull();
}

const TRIALS = 16;
/** How each trial ended, so a pass that never picked anything cannot hide. */
const outcomes = { empty: 0, nothingMatched: 0, picked: 0 };

describe.each(['ur', 'en'] as const)(
  'keys in the buyer cell and the cells beside it (%s)',
  (language) => {
    beforeEach(() => {
      resetClientState(language);
      signedIn(sessionUser({ language }));
    });

    it.each(Array.from({ length: TRIALS }, (_, i) => i + 1))(
      'random typing, deleting and Enter, seed %i',
      async (seed) => {
        const random = rng(seed * 7 + (language === 'ur' ? 1 : 2));
        const answerMs = [0, 80, 250, 500][random.below(4)] ?? 0;
        const { user, sales, backend, quick } = await openWithSlowSearch(language, answerMs);
        let buffer = '';
        const steps: string[] = [];
        const operations = 3 + random.below(6);
        for (let n = 0; n < operations; n += 1) {
          const op = random.below(9);
          if (op <= 2) {
            const text = random.pick(TEXTS);
            steps.push(`type ${text}`);
            await user.keyboard(text);
            buffer += text;
          } else if (op === 3) {
            steps.push('Backspace');
            await user.keyboard('{Backspace}');
            buffer = Array.from(buffer).slice(0, -1).join('');
          } else if (op === 4) {
            steps.push('Delete');
            await user.keyboard('{Delete}');
          } else if (op === 5) {
            steps.push('clear');
            await user.keyboard('{Control>}a{/Control}{Backspace}');
            buffer = '';
          } else if (op === 6) {
            steps.push('wait');
            await wait(random.below(300));
          } else if (op === 7) {
            // Backspace and Delete on the cells beside it, then back to the buyer cell.
            const field = random.pick(['qty', 'rate', 'buyerTotal', 'note'] as const);
            steps.push(`Backspace/Delete in ${field}`);
            try {
              await user.click(cellOf(sales, 0, field));
              await user.keyboard('{Backspace}{Delete}');
            } catch {
              // The cell is not drawn (a column hidden at this size): nothing to press on.
            }
            noLeaveBox();
            await user.click(cellOf(sales, 0, 'buyer'));
            buffer = '';
            await wait(60);
          } else {
            steps.push('wait for the answer');
            await wait(answerMs + 100);
          }
          noLeaveBox();
          expect(
            screen.getByRole('heading', {
              level: 1,
              name: message(language, 'glossary-bikri-book'),
            }),
          ).toBeInTheDocument();
        }
        // The pick: Enter, and give the slow answer time to land.
        steps.push(`Enter with "${buffer}"`);
        const before = backend.previews.at(-1)?.lots[0]?.sales[0]?.buyer_account_id ?? null;
        await user.keyboard('{Enter}');
        await wait(answerMs + 700);
        noLeaveBox();
        const hits = buffer.trim() === '' ? [] : await answerFor(quick, buffer);
        await waitFor(() => {
          const draft = backend.previews.at(-1);
          const picked = draft?.lots[0]?.sales[0]?.buyer_account_id ?? null;
          const context = `steps: ${steps.join(' | ')}; delay ${String(answerMs)} ms; buffer "${buffer}"`;
          if (buffer.trim() === '') {
            // Rule 2: nothing typed, nothing picked.
            expect(picked, `Enter on an empty cell picked ${String(picked)}. ${context}`).toBe(
              before,
            );
            expect(textOf(sales, 0, 'buyer'), context).toBe('');
            outcomes.empty += 1;
          } else if (hits.length === 0) {
            expect(picked, `text that matches nothing picked ${String(picked)}. ${context}`).toBe(
              before,
            );
            outcomes.nothingMatched += 1;
          } else {
            // Rule 3: the first row of the answer to THIS text, not of another.
            const first = hits[0];
            expect(
              picked,
              `Enter picked ${String(picked)}, the answer to "${buffer}" starts with ${String(first?.id)}. ${context}`,
            ).toBe(first?.id);
            // Rule 4: the cell shows his name.
            const shown = textOf(sales, 0, 'buyer');
            const who = BUYERS.find((b) => b.id === picked);
            expect(who, context).toBeDefined();
            if (who !== undefined) expect(shown, context).toBe(NAME(language, who));
            outcomes.picked += 1;
          }
        }, SLOW_MS);
      },
      60_000,
    );
  },
);

afterAll(() => {
  process.stdout.write(
    `picker-keys-fuzz: Enter on an empty cell ${String(outcomes.empty)}, on text that matched nothing ${String(outcomes.nothingMatched)}, picked ${String(outcomes.picked)}\n`,
  );
  expect(outcomes.picked).toBeGreaterThanOrEqual(6);
  expect(outcomes.empty).toBeGreaterThanOrEqual(3);
});
