// The dead amount on the dues reports and the ledger (plan 40, PRD FR-10.10, FR-7.38): R20, R24 and
// R32 have the button "ڈوبی رقم میں ڈالیں" for the row that has focus (dimmed until one has), and it
// opens the real dialog for that person; the heading of a ledger (R13) of a person in the dead amount
// says ڈوبی رقم with the date and the reason, and offers "دوبارہ فعال کریں". In the Urdu and in the
// English app, against a fake backend with the backend's own R20 and R13 answers.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ReportPage } from '../../src/bindings/ReportPage';
import * as mastersApi from '../../src/features/masters/api';
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
import { ARSHAD_DEAD, ARSHAD_PREVIEW } from '../helpers/dead';
import { MARKED_ARSHAD } from '../helpers/dead-real';
import { message, visible } from '../helpers/render';
import { R13_AKRAM, R20_DUES, fakeReports } from '../helpers/reports';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');

const SLOW_MS = { timeout: 8_000 };

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  vi.mocked(mastersApi.deadPreview).mockResolvedValue(ARSHAD_PREVIEW);
  vi.mocked(mastersApi.deadMark).mockResolvedValue(MARKED_ARSHAD);
  vi.mocked(mastersApi.deadUndo).mockResolvedValue({ ok: true });
});

afterEach(() => {
  leaveAppWindow();
});

/** The first party cell of a row of a report page, as the screen finds it. */
function partyOf(page: ReportPage, rowIndex: number) {
  for (const cell of page.rows[rowIndex]?.cells ?? []) {
    if (cell.type === 'party') return cell.party;
  }
  throw new Error('no party in the row');
}

describe.each(LANGUAGES)('the dead amount on the reports (%s)', (language: Language) => {
  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const named = (text: string) => (name: string) => {
    const seen = visible(name);
    return seen === text || seen.startsWith(`${text} `);
  };

  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  async function open(path: string, pages: Parameters<typeof fakeReports>[0] = {}) {
    fakeReports(pages);
    const view = await renderApp(path);
    return { ...view, user: userEvent.setup() };
  }

  it('R20 has the button, dimmed until a row has focus; then it opens the dialog for that person', async () => {
    const { user } = await open('/reports/R20');
    const button = await screen.findByRole(
      'button',
      { name: named(m('dead-move-button')) },
      SLOW_MS,
    );
    expect(button).toBeDisabled();
    await waitFor(() => {
      expect(document.querySelector('.ag-row[row-id="0"]')).not.toBeNull();
    }, SLOW_MS);
    const first = document.querySelector<HTMLElement>('.ag-row[row-id="0"] .ag-cell');
    if (first === null) throw new Error('no cell');
    await user.click(first);
    await waitFor(() => {
      expect(button).toBeEnabled();
    });
    await user.click(button);
    const dialog = await screen.findByRole('dialog');
    const party = partyOf(R20_DUES, 0);
    expect(mastersApi.deadPreview).toHaveBeenCalledWith({
      account_id: party.account_id,
      pukhta_person_key: party.person_key,
    });
    expect(visible(dialog.textContent)).toContain(
      language === 'ur' ? party.name_ur : party.name_en,
    );
    // Moving him asks every screen to read again: the report is asked for a second time.
    const asked = vi.mocked(reportsApi.reportsRun).mock.calls.length;
    await within(dialog).findByText('18,684');
    await user.click(within(dialog).getByRole('button', { name: named(m('dead-move-confirm')) }));
    await waitFor(() => {
      expect(mastersApi.deadMark).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(vi.mocked(reportsApi.reportsRun).mock.calls.length).toBeGreaterThan(asked);
    }, SLOW_MS);
  });

  it.each(['R24', 'R32'])('%s, a dues report, has the button too', async (id) => {
    await open(`/reports/${id}`);
    await screen.findByRole('button', { name: named(m('dead-move-button')) }, SLOW_MS);
  });

  it('a report that is not about people who owe has no such button', async () => {
    await open('/reports/R18');
    await screen.findByRole('heading', { level: 1 }, SLOW_MS);
    expect(screen.queryByRole('button', { name: named(m('dead-move-button')) })).toBeNull();
  });

  it('a ledger of a person in the dead amount says so under its title, with a way back', async () => {
    const page: ReportPage = {
      ...R13_AKRAM,
      filters: {
        ...R13_AKRAM.filters,
        party:
          R13_AKRAM.filters.party === null
            ? null
            : { ...R13_AKRAM.filters.party, dead: ARSHAD_DEAD },
      },
    };
    const { user } = await open('/reports/R13?party=20', { pages: { R13: page } });
    const line = await screen.findByText(
      (_, element) =>
        element?.tagName === 'SPAN' &&
        visible(element.textContent) ===
          m('dead-info', { date: '11-10-2026', reason: m('dead-reason-died') }),
      undefined,
      SLOW_MS,
    );
    expect(line).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: named(m('dead-undo-button')) }));
    const dialog = await screen.findByRole('alertdialog');
    await user.type(within(dialog).getByRole('combobox'), 'Paid');
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(mastersApi.deadUndo).toHaveBeenCalledWith({
        mark_id: ARSHAD_DEAD.mark_id,
        reason: 'Paid',
      });
    });
  });

  it('a ledger of a person who is not in the dead amount has no such line', async () => {
    await open('/reports/R13?party=20');
    await waitFor(() => {
      expect(document.querySelector('.ag-row[row-id="0"]')).not.toBeNull();
    }, SLOW_MS);
    expect(document.querySelector('[data-dead-info]')).toBeNull();
  });
});
