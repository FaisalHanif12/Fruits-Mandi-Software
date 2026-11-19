// The line above the table of R13 and R14 for the balance at the start of a period that starts after the
// account's opening date (PRD FR-19.16, plan 80 R5-4). The backend names it by the message ID
// `report-period-start-balance`, so that it is not called ابتدائی بقایا (the account's own opening row). The
// screen must show the words of that message and never the ID. The opening line of a period that holds the
// opening date keeps its own words. In the Urdu and in the English app, against the backend's R13 answer.

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { message, visible } from '../helpers/render';
import { R13_AKRAM, fakeReports } from '../helpers/reports';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');

const PERIOD_START = 'report-period-start-balance';
const OPENING = 'report-opening-balance';

/** The backend's R13 answer for a period that starts after the opening date: its first line is the period's. */
const R13_LATER_PERIOD = {
  ...R13_AKRAM,
  summary: R13_AKRAM.summary.map((line) =>
    line.label_id === OPENING ? { ...line, label_id: PERIOD_START } : line,
  ),
};

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the opening lines of a ledger (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string) => message(language, id);
  const label = (id: string) => (name: string) => visible(name) === m(id);

  it('R13 with a period after the opening date says "previous balance" for its first line, not the message id', async () => {
    fakeReports({ pages: { R13: R13_LATER_PERIOD } });
    await renderApp('/reports/R13?party=21&from=05-10-2026&to=10-10-2026');
    // The words of the line, with the backend's figure and its side.
    const words = await screen.findByText(label(PERIOD_START), undefined, { timeout: 8_000 });
    expect(visible(words.parentElement?.textContent)).toContain(
      language === 'ur' ? 'پچھلا بقایا' : 'Previous Balance',
    );
    await waitFor(() => {
      expect(vi.mocked(reportsApi.reportsRun)).toHaveBeenCalled();
    });
    // The id itself is never on the screen, and the line is not called the account's opening.
    expect(visible(document.body.textContent)).not.toContain(PERIOD_START);
    expect(screen.queryByText(label(OPENING))).toBeNull();
  });

  it('R13 for a period that holds the opening date keeps ابتدائی بقایا / Opening balance for its first line', async () => {
    fakeReports({ pages: { R13: R13_AKRAM } });
    await renderApp('/reports/R13?party=21');
    expect(await screen.findByText(label(OPENING), undefined, { timeout: 8_000 })).toBeTruthy();
    expect(screen.queryByText(label(PERIOD_START))).toBeNull();
  });

  it('the period-start line is a glossary word in both languages, and not the opening word', () => {
    expect(m(PERIOD_START)).toBe(m('glossary-sabiqa-raqam'));
    expect(m(PERIOD_START)).not.toBe(m(OPENING));
  });
});
