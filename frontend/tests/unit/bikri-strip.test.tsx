// The key numbers of the sale book, docked under its grids in compact and normal windows (plan 12,
// plan 38): the lot on screen or the whole sale (خام بکری, کل اخراجات, صافی بکری, the buyers'
// bills) on one line, and the cash note, straight from the preview. It sits under the scrolling area,
// never over a grid, and holds nothing the keyboard can reach. In a wide window the summary has its
// own column and the strip is not drawn. Urdu and English.

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as hooks from '../../src/shared/hooks/useLayoutStep';
import * as urduApi from '../../src/shared/urdu/api';
import { money } from '../helpers/accounts';
import { previewFor } from '../helpers/bikri-data';
import {
  named,
  openBook,
  salesGrid,
  textOf,
  typeHeaderAndLot,
  typeTwoLines,
} from '../helpers/bikri-screen';
import {
  fail,
  healthyBackend,
  leaveAppWindow,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { message, visible } from '../helpers/render';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');
// The window width is not known in a test: each test says which layout step it is in.
vi.mock('../../src/shared/hooks/useLayoutStep', async (importOriginal) => {
  const actual = await importOriginal<typeof hooks>();
  return { ...actual, useLayoutStep: vi.fn(() => 'normal' as const) };
});

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(hooks.useLayoutStep).mockReturnValue('normal');
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

const strip = (language: Language) =>
  screen.findByRole('region', { name: named(message(language, 'bikri-strip-label')) });

const countOf = (text: string, part: string) => text.split(part).length - 1;

describe.each(LANGUAGES)('the numbers strip (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('shows the four numbers of the sale on one line, from the preview; one lot needs no name', async () => {
    const { user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    await waitFor(() => {
      expect(textOf(salesGrid(language), 1, 'buyerTotal')).toBe('18,684');
    });
    const text = visible((await strip(language)).textContent);
    // The four numbers of Event 1 (PRD 4.4), once each: with one lot the lot is the whole sale, so
    // the strip does not repeat them under two names.
    for (const number of ['29,400', '10,084', '19,316', '31,564']) {
      expect(countOf(text, number)).toBe(1);
    }
    for (const id of [
      'glossary-kham-bikri',
      'bikri-summary-deductions',
      'glossary-safi-bikri',
      'bikri-summary-buyer-total',
    ]) {
      expect(countOf(text, message(language, id))).toBe(1);
    }
    expect(text).not.toContain(message(language, 'bikri-summary-whole'));
  });

  it('is docked under the scrolling area, holds nothing the keyboard can reach, and is not a live region', async () => {
    const { user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    const box = await strip(language);
    // Docked, not floating over the grids: it takes its own room and never covers a cell.
    expect(box).not.toHaveClass('sticky');
    expect(box).toHaveClass('shrink-0');
    // No button, link, field or tab stop inside: Tab and the arrow keys pass over it.
    expect(box.querySelectorAll('button, a, input, select, textarea, [tabindex]')).toHaveLength(0);
    expect(box).not.toHaveAttribute('aria-live');
    expect(box.querySelector('[role="status"], [role="alert"]')).toBeNull();
  });

  it('shows the cash note when cash would go below zero', async () => {
    const { user } = await openBook(language, (draft) => ({
      ...previewFor(draft),
      cash: {
        cash_before_paisa: money(2000),
        change_paisa: money(-7000),
        cash_after_paisa: money(-5000),
        below_zero: true,
        lowers_cash: true,
        needs_confirm: true,
      },
    }));
    await typeHeaderAndLot(user, language);
    const text = visible((await strip(language)).textContent);
    expect(text).toContain(message(language, 'common-cash-after', { amount: '-5,000' }));
  });

  it('says so for a lot that cannot be worked out yet', async () => {
    const { user } = await openBook(language, (draft) => {
      const preview = previewFor(draft);
      for (const lot of preview.lots) lot.totals = null;
      return preview;
    });
    await typeHeaderAndLot(user, language);
    const text = visible((await strip(language)).textContent);
    expect(text).toContain(message(language, 'bikri-strip-incomplete'));
  });

  it.each(['compact', 'normal'] as const)('is drawn in a %s window', async (step) => {
    vi.mocked(hooks.useLayoutStep).mockReturnValue(step);
    const { user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    expect(await strip(language)).toBeInTheDocument();
  });

  it('is not drawn on an untouched screen, nor when the numbers cannot be worked out (empty and error states)', async () => {
    let failing = true;
    const { user } = await openBook(language, (draft) =>
      failing ? Promise.reject(fail({ code: 'ERR_NOT_IMPLEMENTED' })) : previewFor(draft),
    );
    // Empty: nothing was typed, nothing was asked of the backend, so there are no numbers.
    expect(
      screen.queryByRole('region', { name: named(message(language, 'bikri-strip-label')) }),
    ).not.toBeInTheDocument();
    // Error: the numbers could not be worked out. The screen says so; the strip shows none.
    await user.keyboard('LES-1234');
    expect(
      await screen.findByText(
        (text) => visible(text) === message(language, 'bikri-preview-failed'),
      ),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole('region', { name: named(message(language, 'bikri-strip-label')) }),
    ).not.toBeInTheDocument();
    // Data: once the backend answers, the strip is there.
    failing = false;
    await user.click(
      screen.getByRole('button', { name: new RegExp(message(language, 'common-retry')) }),
    );
    expect(await strip(language)).toBeInTheDocument();
  });

  it('a language switch redraws its words in the other language with the same numbers', async () => {
    const { user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    await waitFor(() => {
      expect(textOf(salesGrid(language), 1, 'buyerTotal')).toBe('18,684');
    });
    const next: Language = language === 'ur' ? 'en' : 'ur';
    await user.keyboard('{Alt>}l{/Alt}');
    const text = visible((await strip(next)).textContent);
    expect(text).toContain(message(next, 'glossary-kham-bikri'));
    for (const number of ['29,400', '10,084', '19,316', '31,564']) {
      expect(countOf(text, number)).toBe(1);
    }
  });

  it('is not drawn in a wide window, where the summary has its own column', async () => {
    vi.mocked(hooks.useLayoutStep).mockReturnValue('wide');
    const { user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await screen.findByRole('heading', { name: named(message(language, 'bikri-summary-title')) });
    expect(
      screen.queryByRole('region', { name: named(message(language, 'bikri-strip-label')) }),
    ).not.toBeInTheDocument();
  });
});
