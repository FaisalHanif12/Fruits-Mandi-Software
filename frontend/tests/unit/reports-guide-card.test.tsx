// The guide card of a report (plan 75 T7, PRD FR-19.14), in Urdu and English: pointing at a report on the
// reports page, or moving to it with the keyboard, opens a card beside it with the seven parts (its name,
// ID and group; what it shows; its columns; its filters; what can be done; when to use it; a button that
// opens it). The columns and the filters are the report's own, the card needs no call to the backend,
// Esc closes it, and the words exist for every report in both languages.

import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ReportCatalogEntry } from '../../src/bindings/ReportCatalogEntry';
import * as reportsApi from '../../src/features/reports/api';
import {
  columnTitleIdsOf,
  detailIdOf,
  filterLabelIdsOf,
  specialPrintIdOf,
  whenIdOf,
} from '../../src/features/reports/catalog';
import { ReportGuideCard } from '../../src/features/reports/components/ReportGuideCard';
import { bundles, LANGUAGES, type Language } from '../../src/i18n';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { message, renderIn, visible } from '../helpers/render';
import { BACKEND_CATALOG, fakeReports } from '../helpers/reports';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');

const SLOW_MS = { timeout: 8_000 };

/**
 * One catalogue entry exactly as the backend sent it (the fixture is the backend's own copy, so no
 * stand-in can drift from it). A test that needs other columns passes them.
 */
function entryOf(id: string, columns?: string[]): ReportCatalogEntry {
  const found = BACKEND_CATALOG.find((candidate) => candidate.id === id);
  if (found === undefined) throw new Error(`no ${id} in the catalogue`);
  return columns === undefined ? found : { ...found, column_title_ids: columns };
}

/** A text matcher that finds an element by ALL its own text, ignoring the marks Fluent puts round a term. */
const exactly = (text: string) => (content: string) => visible(content) === text;

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  window.localStorage.clear();
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the words of the guide card (%s)', (language: Language) => {
  const bundle = bundles[language];
  const has = (id: string) => bundle.getMessage(id) !== undefined;

  it('has "what it shows" and "when to use it" for every report the backend lists', () => {
    for (const entry of BACKEND_CATALOG) {
      expect(has(detailIdOf(entry)), `${language}: ${detailIdOf(entry)}`).toBe(true);
      expect(has(whenIdOf(entry)), `${language}: ${whenIdOf(entry)}`).toBe(true);
    }
  });

  it('has the words of every filter box a report has', () => {
    for (const entry of BACKEND_CATALOG) {
      for (const id of filterLabelIdsOf(entry)) {
        expect(has(id), `${language}: ${entry.id} filter ${id}`).toBe(true);
      }
    }
  });

  it('has the words of the special print of the reports that have one', () => {
    for (const entry of BACKEND_CATALOG) {
      const id = specialPrintIdOf(entry);
      if (id !== null) expect(has(id), `${language}: ${id}`).toBe(true);
    }
    expect(
      BACKEND_CATALOG.filter((entry) => specialPrintIdOf(entry) !== null).map((entry) => entry.id),
    ).toEqual(['R1', 'R5', 'R35']);
  });
});

describe('the columns the backend sends', () => {
  it('are all words in both languages, when the backend sends them', () => {
    for (const entry of BACKEND_CATALOG) {
      for (const id of columnTitleIdsOf(entry)) {
        for (const language of LANGUAGES) {
          expect(
            bundles[language].getMessage(id),
            `${language}: ${entry.id} column ${id}`,
          ).toBeDefined();
        }
      }
    }
  });
});

describe.each(LANGUAGES)('the card of one report (%s)', (language: Language) => {
  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);

  function draw(entry: ReportCatalogEntry, onOpen = vi.fn()) {
    const view = renderIn(language, <ReportGuideCard entry={entry} onOpen={onOpen} />);
    return { onOpen, unmount: view.unmount };
  }

  /** The words under one label of the card (a label is a `dt`, its words the `dd` after it). */
  function under(labelId: string): HTMLElement {
    const label = screen.getByText((text) => visible(text) === m(labelId), { selector: 'dt' });
    const words = label.nextElementSibling;
    if (!(words instanceof HTMLElement)) throw new Error(`nothing under ${labelId}`);
    return words;
  }

  it('shows its name, ID and group, what it shows, and when to use it', () => {
    const entry = entryOf('R20');
    draw(entry);
    expect(
      screen.getByRole('heading', {
        level: 3,
        name: (name) => visible(name) === m(entry.title_id),
      }),
    ).toBeInTheDocument();
    expect(screen.getByText(entry.id)).toBeInTheDocument();
    expect(
      screen.getByText((text) => visible(text) === m(entry.group_title_id)),
    ).toBeInTheDocument();
    expect(visible(under('report-guide-shows').textContent)).toBe(m(detailIdOf(entry)));
    expect(visible(under('report-guide-when').textContent)).toBe(m(whenIdOf(entry)));
  });

  it('lists the columns the report sends, and only those', () => {
    const columns = ['report-col-date', 'report-col-debit'];
    draw(entryOf('R11', columns));
    const chips = within(under('report-guide-columns')).getAllByRole('listitem');
    expect(chips.map((chip) => visible(chip.textContent))).toEqual(columns.map((id) => m(id)));
  });

  it('shows, for every report of the catalogue, exactly the columns it sends, in the order it sends them', () => {
    for (const entry of BACKEND_CATALOG) {
      const { unmount } = draw(entry);
      const chips = within(under('report-guide-columns')).getAllByRole('listitem');
      expect(
        chips.map((chip) => visible(chip.textContent)),
        `${language}: ${entry.id}`,
      ).toEqual(entry.column_title_ids.map((id) => m(id)));
      unmount();
    }
  });

  it('leaves out the columns part when the report sends none', () => {
    draw(entryOf('R11', []));
    expect(screen.queryByText((text) => visible(text) === m('report-guide-columns'))).toBeNull();
  });

  it('lists the filters the report takes, and says when there are none', () => {
    const entry = entryOf('R11');
    draw(entry);
    const chips = within(under('report-guide-filters')).getAllByRole('listitem');
    expect(chips.map((chip) => visible(chip.textContent))).toEqual(
      filterLabelIdsOf(entry).map((id) => m(id)),
    );
  });

  it('says there are no filters for a report that takes none', () => {
    draw(entryOf('R19'));
    expect(visible(under('report-guide-filters').textContent)).toBe(m('report-guide-filters-none'));
  });

  it('says an account must be chosen first for a report that needs one', () => {
    draw(entryOf('R13'));
    expect(screen.getByText(exactly(m('report-guide-party-required')))).toBeInTheDocument();
  });

  it('does not say it for a report that opens without one', () => {
    draw(entryOf('R20'));
    expect(screen.queryByText(exactly(m('report-guide-party-required')))).toBeNull();
  });

  it('says what can be done with every report: screen, print, PDF and Excel', () => {
    draw(entryOf('R20'));
    const actions = within(under('report-guide-actions'));
    for (const id of [
      'report-guide-action-screen',
      'report-guide-action-print',
      'report-guide-action-pdf',
      'report-guide-action-excel',
    ]) {
      expect(actions.getByText(exactly(m(id)))).toBeInTheDocument();
    }
  });

  it.each(['R1', 'R5', 'R35'])('adds the special print of %s', (id) => {
    draw(entryOf(id));
    const actions = within(under('report-guide-actions'));
    expect(
      actions.getByText(exactly(m(`report-guide-special-${id.toLowerCase()}`))),
    ).toBeInTheDocument();
  });

  it('opens the report from its button', async () => {
    const { onOpen } = draw(entryOf('R20'));
    await userEvent.setup().click(screen.getByRole('button', { name: m('report-guide-open') }));
    expect(onOpen).toHaveBeenCalledTimes(1);
  });
});

describe.each(LANGUAGES)('the guide card on the reports page (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);

  async function open() {
    fakeReports();
    await renderApp('/reports');
    await screen.findByRole('heading', {
      level: 1,
      name: (name) => visible(name) === m('reports-title'),
    });
    // The catalogue is read from the backend: the cards come after the title.
    await waitFor(() => {
      expect(cardOf('R5')).toBeDefined();
    }, SLOW_MS);
    return userEvent.setup();
  }

  const cardOf = (id: string) =>
    screen
      .queryAllByRole('link')
      .find((link) => link.getAttribute('href')?.endsWith(`/reports/${id}`) === true);

  function link(id: string): HTMLElement {
    const found = cardOf(id);
    if (found === undefined) throw new Error(`no card for ${id}`);
    return found;
  }

  /** The guide cards that are open now: Radix puts `data-side` on the card it places beside a control. */
  const guides = () => Array.from(document.querySelectorAll('[role="group"][data-side]'));

  it('says at the top that pointing at a report shows what it holds', async () => {
    await open();
    expect(
      await screen.findByText((text) => visible(text) === m('reports-hover-hint')),
    ).toBeInTheDocument();
  });

  it('shows the ID of each report as a badge on its card', async () => {
    await open();
    expect(within(link('R5')).getByText('R5')).toBeInTheDocument();
  });

  it('does not open the card of the first report when the page opens, also after a key (Enter on the menu)', async () => {
    fireEvent.keyDown(document.body, { key: 'Enter' });
    await open();
    expect(document.activeElement?.getAttribute('data-nav-item')).not.toBeNull();
    await new Promise((done) => setTimeout(done, 500));
    expect(guides()).toHaveLength(0);
  });

  it('opens a card beside the report that is pointed at, with no call to the backend', async () => {
    const user = await open();
    const callsBefore = vi.mocked(reportsApi.reportsRun).mock.calls.length;
    await user.hover(link('R5'));
    const card = await screen.findByRole(
      'group',
      {
        name: (name) => visible(name) === m('report-guide-label', { title: m('report-r5-title') }),
      },
      SLOW_MS,
    );
    const text = visible(card.textContent);
    expect(text).toContain(m('report-r5-detail'));
    expect(text).toContain(m('report-r5-when'));
    expect(text).toContain(m('report-guide-special-r5'));
    expect(vi.mocked(reportsApi.reportsRun).mock.calls.length).toBe(callsBefore);
  });

  it('opens the report from the button of the card, the same as a click on the small card', async () => {
    const user = await open();
    await user.hover(link('R20'));
    const button = await screen.findByRole(
      'button',
      { name: (name) => visible(name) === m('report-guide-open') },
      SLOW_MS,
    );
    await user.click(button);
    await screen.findByRole('heading', {
      level: 1,
      name: (name) => visible(name) === m('report-r20-title'),
    });
  });

  it('opens at once when a key moves the focus to a report, shows one card at a time, and Esc closes it', async () => {
    const user = await open();
    link('R2').focus();
    await user.keyboard('{ArrowDown}');
    const focused = document.activeElement;
    expect(focused?.tagName).toBe('A');
    await waitFor(() => {
      expect(guides()).toHaveLength(1);
    }, SLOW_MS);
    await user.keyboard('{Escape}');
    expect(guides()).toHaveLength(0);
    // Esc closed the card and nothing else: the reports page is still open and the focus is where it was.
    expect(
      screen.getByRole('heading', {
        level: 1,
        name: (name) => visible(name) === m('reports-title'),
      }),
    ).toBeInTheDocument();
    expect(document.activeElement).toBe(focused);
  });
});
