// D8 Bug 1 (plan 61): a complete sale line shows no figures. Umar's exact case, with the real
// screen, the real grid and the real store, typed key by key: a lot of 200 آم, and three lines,
// مانی 20 at 1,000, انس 50 at 900 and ہارون 130 with the rate typed as 900. The grey hint under the
// empty rate cell of line 3 is also 900 (the rate of the line before it, PRD FR-7.53). Typing the
// same number as the hint must still be a typed rate: the line is then complete, shows its figures
// and counts in the summary. The numbers are D8 Section 3 (the backend's, see helpers/bikri-umar.ts).

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { realAnswer } from '../helpers/bikri-d8-real';
import { openUmarsBook, seedUmarsLot, umarAnswer } from '../helpers/bikri-umar';
import {
  cellOf,
  focusedIn,
  lineDetails,
  salesGrid,
  textOf,
  type User,
} from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { EMPTY_PAIRS } from '../helpers/paged';

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

describe.each(LANGUAGES)('Umar’s three lines on a lot of 200 (%s)', (language) => {
  /** Holds the answer for a draft with buyer 3 and no rate until the test lets it go. */
  function holdLine3() {
    const held = { open: false, waiting: [] as (() => void)[] };
    const answer = (draft: Parameters<typeof umarAnswer>[0]) => {
      const sale = draft.lots[0]?.sales[2];
      const wait = !held.open && sale?.buyer_account_id != null && sale.seller_rate === '';
      const reply = umarAnswer(draft);
      return wait
        ? new Promise<typeof reply>((resolve) => {
            held.waiting.push(() => {
              resolve(reply);
            });
          })
        : reply;
    };
    const release = () => {
      held.open = true;
      held.waiting.splice(0).forEach((go) => {
        go();
      });
    };
    return { answer, release };
  }

  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  /** Types line 1 and line 2, as Umar did, and stops with the cursor on the buyer of line 3. */
  async function typeFirstTwoLines(answer?: Parameters<typeof openUmarsBook>[1]) {
    seedUmarsLot();
    const view = await openUmarsBook(language, answer);
    const sales = salesGrid(language);
    await waitFor(() => {
      expect(cellOf(sales, 0, 'buyer')).toBeInTheDocument();
    }, SLOW_MS);
    await view.user.click(cellOf(sales, 0, 'buyer'));
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:buyer');
    }, SLOW_MS);
    await pickBuyer(view.user, '0');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:qty');
    }, SLOW_MS);
    await view.user.keyboard('20{Enter}1000{Enter}{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('1:buyer');
    }, SLOW_MS);
    await pickBuyer(view.user, '1');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('1:qty');
    }, SLOW_MS);
    await view.user.keyboard('50{Enter}900{Enter}{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('2:buyer');
    }, SLOW_MS);
    return { ...view, sales };
  }

  it('line 3, with the rate typed as 900 (the same as the hint), is complete and shows its figures', async () => {
    const { user, backend, sales } = await typeFirstTwoLines();
    await pickBuyer(user, '2');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('2:qty');
    }, SLOW_MS);
    // The quantity starts as what the lot has left, and the rate cell shows the hint as grey text.
    await waitFor(() => {
      expect(textOf(sales, 2, 'qty')).toBe('130');
    }, SLOW_MS);
    await waitFor(() => {
      expect(cellOf(sales, 2, 'rate')).toHaveClass('cell-hint');
      expect(textOf(sales, 2, 'rate')).toContain('900');
    }, SLOW_MS);
    await user.keyboard('{Enter}900{Enter}');

    // The typed rate reaches the draft the backend is asked about.
    await waitFor(() => {
      expect(backend.lastDraft().lots[0]?.sales[2]).toMatchObject({
        qty: '130',
        seller_rate: '900',
      });
    }, SLOW_MS);
    // The cell holds a typed value now, not a hint, and the line has its figures.
    await waitFor(() => {
      expect(cellOf(sales, 2, 'rate')).not.toHaveClass('cell-hint');
      expect(textOf(sales, 2, 'buyerTotal')).toBe('125,060');
    }, SLOW_MS);
    // The seller amount is in the grid from 1920 px only, and in the details of the line in every window.
    expect(visible((await lineDetails(language, 3)).textContent)).toContain('117,000');
    // The summary counts all three lines: 182,000 gross, 21,920 deductions, 160,080 net.
    await waitFor(() => {
      const text = visible(document.body.textContent);
      expect(text).toContain('182,000');
      expect(text).toContain('21,920');
      expect(text).toContain('160,080');
      expect(text).toContain('194,520');
    }, SLOW_MS);
  });

  it('the answer for the buyer of line 3 lands while the rate is being typed: the typed 900 stays', async () => {
    const { answer, release } = holdLine3();
    const { user, backend, sales } = await typeFirstTwoLines(answer);
    await pickBuyer(user, '2');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('2:qty');
    }, SLOW_MS);
    await waitFor(() => {
      expect(textOf(sales, 2, 'qty')).toBe('130');
    }, SLOW_MS);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('2:rate');
    }, SLOW_MS);
    // The first key opens the cell; the answer (with the hint 900) lands in the middle of the typing.
    await user.keyboard('9');
    release();
    await new Promise((done) => setTimeout(done, 300));
    await user.keyboard('00{Enter}');
    await waitFor(() => {
      expect(backend.lastDraft().lots[0]?.sales[2]).toMatchObject({
        qty: '130',
        seller_rate: '900',
      });
    }, SLOW_MS);
    await waitFor(() => {
      expect(cellOf(sales, 2, 'rate')).not.toHaveClass('cell-hint');
      expect(textOf(sales, 2, 'buyerTotal')).toBe('125,060');
    }, SLOW_MS);
  });

  /** Stops with line 3 holding a buyer and the quantity left, and no rate: the demo screenshot. */
  async function stopOnLine3WithoutRate() {
    const view = await typeFirstTwoLines();
    await pickBuyer(view.user, '2');
    await waitFor(() => {
      expect(focusedIn(view.sales)).toBe('2:qty');
      expect(textOf(view.sales, 2, 'qty')).toBe('130');
    }, SLOW_MS);
    return view;
  }

  const say = (id: string, vars: Record<string, string | number> = {}) =>
    message(language, id, vars);
  const notesWith = (text: string) =>
    screen.queryAllByRole('alert').filter((note) => visible(note.textContent).includes(text));

  it('the hint under the empty rate is words that say it is the last rate, never a bare number (B1)', async () => {
    const { sales } = await stopOnLine3WithoutRate();
    await waitFor(() => {
      expect(cellOf(sales, 2, 'rate')).toHaveClass('cell-hint');
    }, SLOW_MS);
    // "Last 900" (words and the figure), not the figure alone, and not laid out like a figure.
    expect(textOf(sales, 2, 'rate')).toBe(say('bikri-rate-hint-cell', { rate: '900' }));
    expect(cellOf(sales, 2, 'rate')).not.toHaveClass('cell-hint-number');
  });

  it('a line with a buyer and no rate says in words what is missing; a blank row says nothing (B3)', async () => {
    const { user, sales } = await typeFirstTwoLines();
    // Line 3 is a blank row now (no buyer, no rate): silent. The note of line 2 (its rate was
    // empty a moment ago) goes when the answer for the typed rate arrives.
    await waitFor(() => {
      expect(screen.queryAllByRole('alert')).toHaveLength(0);
    }, SLOW_MS);
    await pickBuyer(user, '2');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('2:qty');
    }, SLOW_MS);
    const words = `${say('bikri-qty-note-line', { line: 3 })} ${say('bikri-line-missing-rate')}`;
    await waitFor(() => {
      expect(notesWith(say('bikri-line-missing-rate'))).toHaveLength(1);
    }, SLOW_MS);
    expect(visible(notesWith(say('bikri-line-missing-rate'))[0]?.textContent)).toBe(words);
    // The note names no other line, and goes away when the rate is typed.
    await user.keyboard('{Enter}900{Enter}');
    await waitFor(() => {
      expect(notesWith(say('bikri-line-missing-rate'))).toHaveLength(0);
      expect(textOf(sales, 2, 'buyerTotal')).toBe('125,060');
    }, SLOW_MS);
  });

  it('a line with no buyer but a rate says the buyer is missing (B3)', async () => {
    const { user, sales } = await typeFirstTwoLines();
    await user.click(cellOf(sales, 2, 'rate'));
    await user.keyboard('900{Enter}');
    await waitFor(() => {
      expect(notesWith(say('bikri-line-missing-buyer'))).toHaveLength(1);
    }, SLOW_MS);
  });

  /**
   * A quantity over the lot (D8 Bug 1, B2): the answers for a draft whose first line is over 200 wait until
   * the test lets them go, as a slow backend does, so the cursor has already moved on when the refusal lands.
   */
  function slowOver() {
    const held = { open: false, waiting: [] as (() => void)[] };
    const answer = (draft: Parameters<typeof umarAnswer>[0]) => {
      const over = Number(draft.lots[0]?.sales[0]?.qty ?? '0') > 200;
      const reply = umarAnswer(draft);
      return over && !held.open
        ? new Promise<typeof reply>((resolve) => {
            held.waiting.push(() => {
              resolve(reply);
            });
          })
        : reply;
    };
    const release = () => {
      held.open = true;
      held.waiting.splice(0).forEach((go) => {
        go();
      });
    };
    const hold = () => {
      held.open = false;
    };
    return { answer, release, hold };
  }

  /** The lot of Umar, line 1 with its buyer, the cursor on its quantity. */
  async function onFirstQty(answer: Parameters<typeof openUmarsBook>[1]) {
    seedUmarsLot();
    const view = await openUmarsBook(language, answer);
    const sales = salesGrid(language);
    await waitFor(() => {
      expect(cellOf(sales, 0, 'buyer')).toBeInTheDocument();
    }, SLOW_MS);
    await view.user.click(cellOf(sales, 0, 'buyer'));
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:buyer');
    }, SLOW_MS);
    await pickBuyer(view.user, '0');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:qty');
    }, SLOW_MS);
    return { ...view, sales };
  }

  const refusal = () => say('err-qty-exceeds-lot', { available: '200' });
  const qtyInput = () => {
    const field = document.activeElement;
    if (!(field instanceof HTMLInputElement)) throw new Error('the cursor is not in a field');
    return field;
  };

  it('a quantity over the lot sends the cursor back to it, selected, when the refusal lands, and keeps it there until fixed (B2)', async () => {
    const { answer, release } = slowOver();
    const { user, sales } = await onFirstQty(answer);
    // 250 on a lot of 200: the value is taken and Enter goes on to the rate, as the refusal is not here yet.
    await user.keyboard('250{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:rate');
    }, SLOW_MS);
    // The refusal lands. Whatever cell the cursor is on, it goes back to the quantity, typing, 250 selected.
    release();
    await waitFor(() => {
      expect(notesWith(refusal())).toHaveLength(1);
      expect(focusedIn(sales)).toBe('0:qty');
      expect(qtyInput().value).toBe('250');
    }, SLOW_MS);
    expect([qtyInput().selectionStart, qtyInput().selectionEnd]).toEqual([0, 3]);
    expect(cellOf(sales, 0, 'qty')).toHaveClass('cell-error');
    // No key takes the cursor off it while the refusal stands: the arrows, Tab and Enter all stay.
    await user.keyboard('{ArrowDown}{ArrowUp}{ArrowRight}{ArrowLeft}{Tab}{Enter}');
    expect(focusedIn(sales)).toBe('0:qty');
    expect(textOf(sales, 0, 'qty')).toBe('250');
    // Another cell of the line takes no input: a click gives the cursor back to the quantity.
    await user.click(cellOf(sales, 0, 'rate'));
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:qty');
    }, SLOW_MS);
    expect(cellOf(sales, 0, 'rate').querySelector('input')).toBeNull();
    // A fixed quantity goes on to the rate as usual, and the hold is gone.
    await user.keyboard('20{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:rate');
    }, SLOW_MS);
    await waitFor(() => {
      expect(notesWith(refusal())).toHaveLength(0);
      expect(cellOf(sales, 0, 'qty')).not.toHaveClass('cell-error');
    }, SLOW_MS);
    // The hold is gone: the arrow that goes back (← in English, → in Urdu) leaves the rate for the quantity.
    await user.keyboard(language === 'ur' ? '{ArrowRight}' : '{ArrowLeft}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:qty');
    }, SLOW_MS);
  });

  it('typing in the rate when the refusal lands: the cursor still goes back, and what was typed is kept (B2)', async () => {
    const { answer, release } = slowOver();
    const { user, sales } = await onFirstQty(answer);
    await user.keyboard('250{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:rate');
    }, SLOW_MS);
    await user.keyboard('1000');
    expect(document.activeElement?.tagName).toBe('INPUT');
    release();
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:qty');
      expect(qtyInput().value).toBe('250');
    }, SLOW_MS);
    expect(textOf(sales, 0, 'rate')).toBe('1,000');
    // Fixed: 20 at 1,000 has its figures.
    await user.keyboard('20{Enter}');
    await waitFor(() => {
      expect(textOf(sales, 0, 'buyerTotal')).toBe('21,360');
    }, SLOW_MS);
  });

  it('a second wrong quantity sends the cursor back again; the same wrong one only once (B2)', async () => {
    const { answer, release } = slowOver();
    const { user, sales } = await onFirstQty(answer);
    await user.keyboard('250{Enter}');
    release();
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:qty');
      expect(qtyInput().value).toBe('250');
    }, SLOW_MS);
    // 260 is also over: Enter goes on (the text changed), the answer lands, and the cursor comes back.
    await user.keyboard('260{Enter}');
    await waitFor(() => {
      expect(qtyInput().value).toBe('260');
      expect(focusedIn(sales)).toBe('0:qty');
    }, SLOW_MS);
    expect(notesWith(refusal())).toHaveLength(1);
    // Esc cancels the typing and the cursor stays on the cell, as always (PRD FR-7.18).
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(document.querySelector('.ag-cell-inline-editing')).toBeNull();
    }, SLOW_MS);
    expect(focusedIn(sales)).toBe('0:qty');
  });

  it('Enter on the quantity that is over stays, and a fixed quantity goes on (B2)', async () => {
    const { user, sales } = await onFirstQty(umarAnswer);
    await user.keyboard('250{Enter}');
    const sentBack = say('err-qty-exceeds-lot', { available: '200' });
    await waitFor(() => {
      expect(notesWith(sentBack)).toHaveLength(1);
      expect(focusedIn(sales)).toBe('0:qty');
    }, SLOW_MS);
    await user.keyboard('{Enter}');
    expect(focusedIn(sales)).toBe('0:qty');
    await user.keyboard('20{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:rate');
    }, SLOW_MS);
  });

  it('with the backend’s own answers: the empty rate says so, then every figure of D8 is on screen', async () => {
    const { user, sales } = await typeFirstTwoLines(realAnswer);
    await pickBuyer(user, '2');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('2:qty');
    }, SLOW_MS);
    // The backend's answer for the line with no rate: the hint 900, no figures, the lot counts 130 as
    // sold (باقی 0), and the line says what it needs.
    await waitFor(() => {
      expect(textOf(sales, 2, 'rate')).toBe(say('bikri-rate-hint-cell', { rate: '900' }));
      expect(notesWith(say('bikri-line-missing-rate'))).toHaveLength(1);
    }, SLOW_MS);
    expect(textOf(sales, 2, 'buyerTotal')).toBe('');
    await user.keyboard('{Enter}900{Enter}');
    await waitFor(() => {
      expect(textOf(sales, 2, 'buyerTotal')).toBe('125,060');
    }, SLOW_MS);
    // The seller amount is in the grid from 1920 px only, and in the details of the line in every window.
    expect(visible((await lineDetails(language, 3)).textContent)).toContain('117,000');
    const text = visible(document.body.textContent);
    for (const figure of ['182,000', '21,920', '160,080', '194,520', '9,376', '7,032']) {
      expect(text).toContain(figure);
    }
    expect(notesWith(say('bikri-line-missing-rate'))).toHaveLength(0);
  });

  it('Enter to line 4, then the blank line 4 removed: line 3 keeps its rate and its figures', async () => {
    const { user, sales } = await typeFirstTwoLines(realAnswer);
    await pickBuyer(user, '2');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('2:qty');
    }, SLOW_MS);
    // Rate, then the paid cell (this buyer may take credit), then the next line.
    await user.keyboard('{Enter}900{Enter}{Enter}');
    await waitFor(() => {
      expect(textOf(sales, 2, 'buyerTotal')).toBe('125,060');
      expect(focusedIn(sales)).toBe('3:buyer');
    }, SLOW_MS);
    // The blank line 4 goes by its remove button; the cursor comes back to line 3.
    const removeButton = sales.querySelector<HTMLElement>(
      '.ag-row[row-index="3"] .row-remove-button',
    );
    if (removeButton === null) throw new Error('no remove button on line 4');
    await user.click(removeButton);
    await waitFor(() => {
      expect(sales.querySelector('.ag-row[row-index="3"]')).toBeNull();
      expect(focusedIn(sales)?.startsWith('2:')).toBe(true);
    }, SLOW_MS);
    expect(textOf(sales, 2, 'rate')).toBe('900');
    expect(cellOf(sales, 2, 'rate')).not.toHaveClass('cell-hint');
    expect(textOf(sales, 2, 'buyerTotal')).toBe('125,060');
    // Typing 900 again on that rate changes nothing and breaks nothing.
    await user.click(cellOf(sales, 2, 'rate'));
    await user.keyboard('900{Enter}');
    await waitFor(() => {
      expect(textOf(sales, 2, 'rate')).toBe('900');
      expect(textOf(sales, 2, 'buyerTotal')).toBe('125,060');
    }, SLOW_MS);
  });

  it('choosing the buyer of line 3 again keeps its rate and its figures', async () => {
    const { user, backend, sales } = await typeFirstTwoLines(realAnswer);
    await pickBuyer(user, '2');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('2:qty');
    }, SLOW_MS);
    await user.keyboard('{Enter}900{Enter}');
    await waitFor(() => {
      expect(textOf(sales, 2, 'buyerTotal')).toBe('125,060');
    }, SLOW_MS);
    await user.click(cellOf(sales, 2, 'buyer'));
    await waitFor(() => {
      expect(document.activeElement?.tagName).toBe('INPUT');
    }, SLOW_MS);
    await user.keyboard('2');
    await screen.findAllByRole('option', undefined, SLOW_MS);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('2:qty');
    }, SLOW_MS);
    expect(backend.lastDraft().lots[0]?.sales[2]).toMatchObject({ qty: '130', seller_rate: '900' });
    expect(textOf(sales, 2, 'rate')).toBe('900');
    expect(textOf(sales, 2, 'buyerTotal')).toBe('125,060');
  });
});
