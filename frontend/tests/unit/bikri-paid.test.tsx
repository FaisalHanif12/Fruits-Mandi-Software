// D8 Feature 4, plan 65 (PRD FR-12.44): the money paid on the spot (موقع پر جمع). The real screen, the
// real grid and the real store, typed key by key, against the backend's answers for Umar's example
// (D8 Feature 4): مانی pays all 21,360, انس pays 10,000 of 48,100, ہارون pays 0. The screen sends the
// paid amount as typed and shows what the backend sends back (the amount of a buyer who must pay in
// full, what is left on the account, the totals); it works out no money (CLAUDE.md rule 1).

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { BikriDraft } from '../../src/bindings/BikriDraft';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import { buildDraft } from '../../src/features/bikri/draft';
import { applyBackendDraft, type DraftRecords } from '../../src/features/bikri/draft-load';
import { createBikriDraft } from '../../src/features/bikri/draft-store';
import { SALE } from '../../src/features/bikri/types';
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
import { chuktaAnswer, paidAnswer } from '../helpers/bikri-paid-real';
import { seedBikriDraft } from '../helpers/bikri-seed';
import { editable } from '../helpers/bikri-plan16';
import { SHAHID } from '../helpers/bikri-saved';
import { seedUmarsLot, umarAnswer } from '../helpers/bikri-umar';
import { EHSAN, UMAR_BUYERS } from '../helpers/bikri-umar-data';
import {
  cellOf,
  focusedIn,
  openBook as openSaleBook,
  salesGrid,
  textOf,
  type User,
} from '../helpers/bikri-screen';
import { fakePartners } from '../helpers/partners';
import { EMPTY_PAIRS } from '../helpers/paged';
import { message, visible } from '../helpers/render';

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

// Each test types whole lines key by key through the real grid: when the whole suite runs at once the
// machine is busy, so a test may take longer than the default 30 seconds.
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

/** Types a short code into the buyer cell the cursor is on, waits for the list, takes the match. */
async function pickBuyer(user: User, code: string) {
  await user.keyboard(code);
  await screen.findAllByRole('option', undefined, SLOW_MS);
  await user.keyboard('{Enter}');
}

/** The Umar lot opened on the real screen, with a چکتا buyer known to the buyer picker. */
async function openBook(
  language: Language,
  answer: (draft: BikriDraft) => BikriPreview | Promise<BikriPreview> = umarAnswer,
) {
  const backend = fakeBikri(answer);
  fakeAccounts([EHSAN, ...UMAR_BUYERS, CHUKTA_KHATA]);
  fakeAccountSearch([EHSAN, ...UMAR_BUYERS, CHUKTA_KHATA], {}, [SHAHID]);
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
  return { ...app, backend, user, sales };
}

/** The rows of the summary: its words and its figure, by the words. */
function summaryFigure(language: Language, id: string): string {
  const summary = document.querySelector<HTMLElement>(
    'section[aria-labelledby="bikri-summary-heading"]',
  );
  if (summary === null) throw new Error('no summary on the screen');
  const label = message(language, id);
  const row = [...summary.querySelectorAll('dl > div')].find(
    (candidate) => visible(candidate.querySelector('dt')?.textContent) === label,
  );
  return visible(row?.querySelector('dd')?.textContent);
}

describe.each(LANGUAGES)('money paid on the spot, by keyboard (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('Umar’s example: 21,360 / 10,000 / 0 paid, left 0 / 38,100 / 125,060, totals 194,520 / 31,360 / 163,160, four Enters a line', async () => {
    const { user, backend, sales } = await openBook(language);
    // Line 1: buyer, qty, rate and the paid cell, four Enters, then the next line's buyer (NFR-USE-6).
    await pickBuyer(user, '0');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:qty');
    }, SLOW_MS);
    await user.keyboard('20{Enter}1000{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:paid');
    }, SLOW_MS);
    await user.keyboard('21360{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('1:buyer');
    }, SLOW_MS);
    await pickBuyer(user, '1');
    await user.keyboard('50{Enter}900{Enter}10000{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('2:buyer');
    }, SLOW_MS);
    await pickBuyer(user, '2');
    await waitFor(() => {
      expect(textOf(sales, 2, 'qty')).toBe('130');
    }, SLOW_MS);
    // ہارون pays nothing: the paid cell is left empty.
    await user.keyboard('{Enter}900{Enter}{Enter}');
    await waitFor(() => {
      expect(textOf(sales, 2, 'buyerTotal')).toBe('125,060');
    }, SLOW_MS);

    // The draft carries what was typed, as text: nothing for the empty cell, and no payment type.
    const draft = backend.lastDraft();
    expect(draft.lots[0]?.sales.slice(0, 3).map((sale) => sale.paid)).toEqual([
      '21360',
      '10000',
      null,
    ]);
    expect('payment_type' in (draft.lots[0]?.sales[0] ?? {})).toBe(false);
    // The cells show the typed amounts; the empty one stays empty.
    expect(textOf(sales, 0, 'paid')).toBe('21,360');
    expect(textOf(sales, 1, 'paid')).toBe('10,000');
    expect(textOf(sales, 2, 'paid')).toBe('');

    // What is left on each account is the backend's number, in the details of the line.
    const left = async (row: number) => {
      await user.click(cellOf(sales, row, 'rate'));
      const details = document.querySelector<HTMLElement>(
        'section[aria-labelledby^="line-details-"]',
      );
      if (details === null) throw new Error('no details of the line');
      const label = within(details).getByText(message(language, 'bikri-line-left'));
      return visible(label.closest('div')?.querySelector('output')?.textContent);
    };
    await waitFor(async () => {
      expect(await left(0)).toBe('0');
    }, SLOW_MS);
    expect(await left(1)).toBe('38,100');
    expect(await left(2)).toBe('125,060');

    // The summary: the buyer total, what stays on the accounts and what was paid on the spot.
    expect(summaryFigure(language, 'bikri-summary-buyer-total')).toBe('194,520');
    expect(summaryFigure(language, 'glossary-udhari')).toBe('163,160');
    expect(summaryFigure(language, 'glossary-naqdi')).toBe('31,360');
  });

  it('a چکتا buyer pays in full: the backend sets the amount, the cell shows it closed, and Enter after the rate goes on (three Enters)', async () => {
    const { user, backend, sales } = await openBook(language);
    await user.keyboard('shahid');
    await screen.findByRole(
      'option',
      { name: (name) => visible(name).includes(language === 'ur' ? 'شاہد' : 'Shahid') },
      SLOW_MS,
    );
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:qty');
    }, SLOW_MS);
    await user.keyboard('20{Enter}1000{Enter}');
    // The answer says the buyer pays in full. The cursor is on the next line's buyer, not on the paid
    // cell: whether the answer was there before Enter or came a moment after it.
    await waitFor(() => {
      expect(textOf(sales, 0, 'paid')).toBe('21,360');
      expect(focusedIn(sales)).toBe('1:buyer');
    }, SLOW_MS);
    // The cell is the backend's amount, drawn as a calculated cell, and cannot be typed in.
    expect(cellOf(sales, 0, 'paid')).toHaveClass('cell-calc');
    await user.click(cellOf(sales, 0, 'paid'));
    await user.keyboard('5');
    expect(sales.querySelector('.ag-cell-inline-editing')).toBeNull();
    expect(textOf(sales, 0, 'paid')).toBe('21,360');
    // Nothing was typed for it: the draft sends no paid amount for this line.
    expect(backend.lastDraft().lots[0]?.sales[0]?.paid).toBeNull();
    // The details say why the cell is closed, and what is left.
    expect(visible(document.body.textContent)).toContain(
      message(language, 'bikri-paid-fixed-note'),
    );
  });

  it('more than the buyer total is refused on the paid cell, with the most allowed, in words', async () => {
    const { user, sales } = await openBook(language);
    await pickBuyer(user, '0');
    await user.keyboard('20{Enter}1000{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:paid');
    }, SLOW_MS);
    await user.keyboard('30000{Enter}');
    // The refusal is on the cell (the error look) and in words: the most that can be paid is 21,360.
    await waitFor(() => {
      expect(cellOf(sales, 0, 'paid')).toHaveClass('cell-error');
    }, SLOW_MS);
    const words = message(language, 'err-paid-exceeds-total', { max: '21,360' });
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(words);
    }, SLOW_MS);
    expect(visible(document.body.textContent)).not.toContain('ERR_');
    // The line keeps its figures, and what is typed stays in the cell to be fixed.
    expect(textOf(sales, 0, 'buyerTotal')).toBe('21,360');
    expect(textOf(sales, 0, 'paid')).toBe('30,000');
    // Fixing it clears the refusal.
    await user.click(cellOf(sales, 0, 'paid'));
    await user.keyboard('21360{Enter}');
    await waitFor(() => {
      expect(cellOf(sales, 0, 'paid')).not.toHaveClass('cell-error');
    }, SLOW_MS);
  });

  it('a language switch in the middle of an entry keeps the typed paid amounts and the cursor', async () => {
    const { user, sales } = await openBook(language);
    await pickBuyer(user, '0');
    await user.keyboard('20{Enter}1000{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:paid');
    }, SLOW_MS);
    await user.keyboard('5000');
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('1:buyer');
    }, SLOW_MS);
    await user.click(cellOf(sales, 0, 'paid'));
    const next: Language = language === 'ur' ? 'en' : 'ur';
    await user.keyboard('{Alt>}l{/Alt}');
    await waitFor(() => {
      expect(textOf(salesGrid(next), 0, 'paid')).toBe('5,000');
    }, SLOW_MS);
    expect(textOf(salesGrid(next), 0, 'buyerTotal')).toBe('21,360');
    expect(focusedIn(salesGrid(next))).toBe('0:paid');
  });
});

describe.each(LANGUAGES)(
  'with the backend’s own answers for money paid on the spot (%s)',
  (language) => {
    beforeEach(() => {
      resetClientState(language);
      signedIn(sessionUser({ language }));
    });

    /** Types Umar's three lines with four Enters each; the third buyer pays nothing. */
    async function typeThreeLines(first: string) {
      const view = await openBook(language, paidAnswer);
      const { user, sales } = view;
      await pickBuyer(user, '0');
      await user.keyboard('20{Enter}1000{Enter}');
      await waitFor(() => {
        expect(focusedIn(sales)).toBe('0:paid');
      }, SLOW_MS);
      await user.keyboard(`${first}{Enter}`);
      await waitFor(() => {
        expect(focusedIn(sales)).toBe('1:buyer');
      }, SLOW_MS);
      await pickBuyer(user, '1');
      await user.keyboard('50{Enter}900{Enter}10000{Enter}');
      await waitFor(() => {
        expect(focusedIn(sales)).toBe('2:buyer');
      }, SLOW_MS);
      await pickBuyer(user, '2');
      await waitFor(() => {
        expect(textOf(sales, 2, 'qty')).toBe('130');
      }, SLOW_MS);
      await user.keyboard('{Enter}900{Enter}{Enter}');
      return view;
    }

    it('E2E-65: the three lines show their paid amounts, what is left and the totals the backend wrote', async () => {
      const { user, sales } = await typeThreeLines('21360');
      await waitFor(() => {
        expect(textOf(sales, 2, 'buyerTotal')).toBe('125,060');
      }, SLOW_MS);
      expect([0, 1, 2].map((row) => textOf(sales, row, 'buyerTotal'))).toEqual([
        '21,360',
        '48,100',
        '125,060',
      ]);
      expect([0, 1, 2].map((row) => textOf(sales, row, 'paid'))).toEqual(['21,360', '10,000', '']);
      const left = async (row: number) => {
        await user.click(cellOf(sales, row, 'rate'));
        const details = document.querySelector<HTMLElement>(
          'section[aria-labelledby^="line-details-"]',
        );
        if (details === null) throw new Error('no details of the line');
        const label = within(details).getByText(message(language, 'bikri-line-left'));
        return visible(label.closest('div')?.querySelector('output')?.textContent);
      };
      expect(await left(0)).toBe('0');
      expect(await left(1)).toBe('38,100');
      expect(await left(2)).toBe('125,060');
      expect(summaryFigure(language, 'bikri-summary-buyer-total')).toBe('194,520');
      expect(summaryFigure(language, 'glossary-udhari')).toBe('163,160');
      expect(summaryFigure(language, 'glossary-naqdi')).toBe('31,360');
    });

    it('one rupee over the buyer total is refused on the paid cell with the most allowed, and the line keeps its figures', async () => {
      const { sales } = await typeThreeLines('21361');
      await waitFor(() => {
        expect(cellOf(sales, 0, 'paid')).toHaveClass('cell-error');
      }, SLOW_MS);
      expect(visible(document.body.textContent)).toContain(
        message(language, 'err-paid-exceeds-total', { max: '21,360' }),
      );
      expect(textOf(sales, 0, 'buyerTotal')).toBe('21,360');
      expect(textOf(sales, 0, 'paid')).toBe('21,361');
      // Counted as 0 until it is fixed: all of it is left on the accounts (the backend's summary, which
      // is the file's once line 3 is in).
      await waitFor(() => {
        expect(summaryFigure(language, 'glossary-udhari')).toBe('194,520');
      }, SLOW_MS);
      expect(summaryFigure(language, 'glossary-naqdi')).toBe('0');
    });

    it('Event 2: a چکتا buyer pays the 12,800 the backend sets, the paid cell is closed, and Enter after the rate goes on', async () => {
      seedBikriDraft({ lot: 'karim' });
      const { user, backend } = await openSaleBook(language, chuktaAnswer);
      const sales = salesGrid(language);
      await waitFor(() => {
        expect(cellOf(sales, 0, 'buyer')).toBeInTheDocument();
      }, SLOW_MS);
      await user.click(cellOf(sales, 0, 'buyer'));
      await user.keyboard('shahid');
      await screen.findByRole(
        'option',
        { name: (name) => visible(name).includes(language === 'ur' ? 'شاہد' : 'Shahid') },
        SLOW_MS,
      );
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(focusedIn(sales)).toBe('0:qty');
      }, SLOW_MS);
      await user.keyboard('10{Enter}1200{Enter}');
      await waitFor(() => {
        expect(textOf(sales, 0, 'paid')).toBe('12,800');
        expect(focusedIn(sales)).toBe('1:buyer');
      }, SLOW_MS);
      expect(cellOf(sales, 0, 'paid')).toHaveClass('cell-calc');
      expect(textOf(sales, 0, 'buyerTotal')).toBe('12,800');
      // Nothing was typed for the paid cell: the draft sends none, and no payment type.
      expect(backend.lastDraft().lots[0]?.sales[0]?.paid).toBeNull();
      expect(summaryFigure(language, 'glossary-naqdi')).toBe('12,800');
    });
  },
);

describe.each(LANGUAGES)('a line kept as it was saved (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('the details say the line is kept as saved, not that the buyer pays in full (plan 65 point 10a)', async () => {
    seedBikriDraft({ lot: 'karim' });
    // The backend's answer for a must-pay buyer's unchanged line of an edit: closed, with the working
    // `working-paid-kept` (its amount can be less than the total).
    const kept = (draft: BikriDraft) => {
      const preview = chuktaAnswer(draft);
      const calc = preview.lots[0]?.lines[0]?.calc;
      if (calc == null) throw new Error('the file has no finished line');
      calc.workings.paid = { ...calc.workings.paid, message_id: 'working-paid-kept' };
      return preview;
    };
    const { user } = await openSaleBook(language, kept);
    const sales = salesGrid(language);
    await waitFor(() => {
      expect(cellOf(sales, 0, 'buyer')).toBeInTheDocument();
    }, SLOW_MS);
    await user.click(cellOf(sales, 0, 'buyer'));
    await user.keyboard('shahid');
    await screen.findAllByRole('option', undefined, SLOW_MS);
    await user.keyboard('{Enter}10{Enter}1200{Enter}');
    await waitFor(() => {
      expect(textOf(sales, 0, 'paid')).toBe('12,800');
    }, SLOW_MS);
    await user.click(cellOf(sales, 0, 'rate'));
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(
        message(language, 'bikri-paid-kept-note'),
      );
    }, SLOW_MS);
    expect(visible(document.body.textContent)).not.toContain(
      message(language, 'bikri-paid-fixed-note'),
    );
  });
});

/** Every id of the draft found as a record named after its id. */
const RECORDS: DraftRecords = {
  account: (id) =>
    id === null
      ? null
      : { id, code: `c${String(id)}`, name_ur: `ur${String(id)}`, name_en: `en${String(id)}` },
  master: (scope, id) =>
    id === null
      ? null
      : {
          id,
          code: `${scope}${String(id)}`,
          name_ur: `ur-${scope}-${String(id)}`,
          name_en: `en-${scope}-${String(id)}`,
        },
};

describe('the paid amount of a saved line (edit)', () => {
  it('comes back into the cell as text and goes out as the same text; a line paid 0 comes back empty and goes out as nothing', () => {
    const draft = structuredClone(editable().draft);
    const sales = draft.lots[0]?.sales;
    const [first, second] = sales ?? [];
    if (first === undefined || second === undefined) throw new Error('the draft has no two lines');
    first.paid = '10000';
    second.paid = null;
    const stores = createBikriDraft('10-10-2026');
    applyBackendDraft(stores, draft, RECORDS);
    const lot = stores.lots.getState().rows[0];
    if (lot === undefined) throw new Error('no lot row');
    const rows = stores.salesFor(lot.rowId).getState().rows;
    expect(rows[0]?.typed[SALE.paid]).toBe('10000');
    expect(rows[1]?.typed[SALE.paid]).toBeUndefined();
    const sent = buildDraft(stores).lots[0]?.sales;
    expect(sent?.[0]?.paid).toBe('10000');
    expect(sent?.[1]?.paid).toBeNull();
  });
});
