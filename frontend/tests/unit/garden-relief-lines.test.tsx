// The relief lines of a garden (PRD FR-12.51, plan 78 D3), in Urdu and English: under the table of R36, the rent the
// garden was made with and the relief from the owner (only when a rent was changed), and in a partner's statement the
// closing line of that garden, which then says the rent now, the rent it was made with and the relief. The figures are
// the backend's own values (the real closing line of E2E-40 with the relief values of its test: rent 45,000, made with
// 50,000, relief 5,000); the screen adds nothing up.

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ReportPage } from '../../src/bindings/ReportPage';
import type { StatementLine } from '../../src/features/partners/types';
import { detailText, type StatementSay } from '../../src/features/partners/statement-text';
import * as reportsApi from '../../src/features/reports/api';
import * as searchApi from '../../src/shared/search/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { fakeAccountSearch } from '../helpers/accounts';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeGardens } from '../helpers/gardens';
import { paisaForTest } from '../helpers/paisa';
import { fakePartners } from '../helpers/partners';
import { PARTNERS_STATEMENT_GARDEN_ALIF } from '../helpers/gardens-real';
import { message, visible } from '../helpers/render';
import { R36_GARDEN, fakeReports } from '../helpers/reports';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/gardens/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');

const SLOW_MS = { timeout: 8_000 };
const rupees = (value: number) => paisaForTest(value * 100);

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners([]);
  fakeGardens();
  fakeAccountSearch([]);
  vi.mocked(searchApi.searchMasters).mockResolvedValue([]);
  vi.mocked(searchApi.searchRecent).mockResolvedValue([]);
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the gardens report R36 with a relief (%s)', (language: Language) => {
  const m = (id: string) => message(language, id);

  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  /** R36 as the backend sends it when a garden's rent was changed: the table, and two lines under it. */
  const withRelief: ReportPage = {
    ...R36_GARDEN,
    summary: [
      { label_id: 'report-r36-rent-original', cell: { type: 'money', paisa: rupees(50_000) } },
      { label_id: 'report-r36-relief', cell: { type: 'money', paisa: rupees(5_000) } },
    ],
  };

  it('shows the rent the garden was made with and the relief, as the backend sent them', async () => {
    fakeReports({ pages: { R36: withRelief } });
    await renderApp('/reports/R36');
    const original = await screen.findByText(
      (text) => visible(text) === m('report-r36-rent-original'),
      undefined,
      SLOW_MS,
    );
    const relief = screen.getByText((text) => visible(text) === m('report-r36-relief'));
    // Each label has its figure next to it, in the same strip.
    expect(visible(original.parentElement?.textContent)).toContain('50,000');
    expect(visible(relief.parentElement?.textContent)).toContain('5,000');
    // The words are said by Fluent in this language, never the message id.
    expect(visible(original.textContent)).not.toContain('report-r36');
  });

  it('shows no such line when no rent was changed (the real answer has no summary)', async () => {
    fakeReports();
    await renderApp('/reports/R36');
    await waitFor(() => {
      expect(vi.mocked(reportsApi.reportsRun)).toHaveBeenCalled();
    }, SLOW_MS);
    // The garden's row is drawn (its money in is 66,400) and nothing under the table says relief.
    await screen.findAllByText((text) => visible(text).includes('66,400'), undefined, SLOW_MS);
    expect(screen.queryByText((text) => visible(text) === m('report-r36-relief'))).toBeNull();
    expect(
      screen.queryByText((text) => visible(text) === m('report-r36-rent-original')),
    ).toBeNull();
  });
});

describe.each(LANGUAGES)('the closing line of a garden in a partner statement (%s)', (language) => {
  const say: StatementSay = {
    language,
    grouping: 'international',
    text: (id, vars = {}) => message(language, id, vars),
    exists: () => true,
  };
  const closing = (): StatementLine => {
    const found = PARTNERS_STATEMENT_GARDEN_ALIF.lines.find(
      (line) => line.detail === 'garden_closed',
    );
    if (found === undefined) throw new Error('the real statement has no closing line');
    return structuredClone(found);
  };

  it('with no relief it is the plain closing sentence: no rent words', () => {
    const text = visible(detailText(closing(), say));
    expect(text).toContain('0080');
    expect(text).toContain('66,400');
    expect(text).not.toContain('50,000');
    expect(text).not.toBe('');
    // No variable is left open and no message id is shown.
    expect(text).not.toContain('$');
    expect(text).not.toContain('partners-statement-detail');
  });

  it('with relief it says the rent now, the rent it was made with and the relief, with the closing figures too', () => {
    const line = closing();
    line.values = {
      ...line.values,
      rent_paisa: 4_500_000,
      rent_original_paisa: 5_000_000,
      relief_paisa: 500_000,
    };
    const text = visible(detailText(line, say));
    for (const figure of ['45,000', '50,000', '5,000', '66,400', '55,000', '11,400']) {
      expect(text, figure).toContain(figure);
    }
    expect(text).toContain('0080');
    expect(text).not.toContain('$');
    expect(text).not.toContain('partners-statement-detail');
    // It is its own sentence, not the plain one with something added at the end.
    expect(text).not.toBe(visible(detailText(closing(), say)));
  });

  it('the words of this language exist for the relief sentence and the two lines of R36', () => {
    expect(
      message(language, 'partners-statement-detail-garden-closed-relief', {
        garden: '0080',
        rent: '45,000',
        rent_original: '50,000',
        relief: '5,000',
        cost: '55,000',
        sales: '66,400',
        result: '11,400',
        share: '60',
        part: '6,840',
        back: '39,840',
      }).trim(),
    ).not.toBe('');
    for (const id of ['report-r36-rent-original', 'report-r36-relief']) {
      expect(message(language, id).trim(), id).not.toBe('');
    }
  });
});
