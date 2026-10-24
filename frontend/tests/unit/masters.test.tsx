// The nine master lists (plan 06, PRD 10.3, FR-7.17, FR-7.24, FR-7.66): the انڈکس menu, one list,
// and the form to add, change and make inactive, in the Urdu and in the English app, by keyboard,
// against a fake backend. One generic screen, so city, item, packing, expense head and the rest
// share these flows.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as mastersApi from '../../src/features/masters/api';
import { MASTER_CONFIGS } from '../../src/features/masters/configs';
import { useMasterFocus } from '../../src/features/masters/store';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import * as searchApi from '../../src/shared/search/api';
import * as urduApi from '../../src/shared/urdu/api';
import {
  fail,
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { account, fakeAccountSearch } from '../helpers/accounts';
import { fakeMasters, record } from '../helpers/masters';
import { message, visible } from '../helpers/render';
import { suggestionsFor } from '../helpers/translit';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const KOT = record('city', 1, '0001', 'کوٹ رادھا کشن', 'Kot Radha Kishan');
const LAHORE = record('city', 2, '0002', 'لاہور', 'Lahore');
const OLD = record('city', 3, '0003', 'قصور', 'Kasur', { is_active: false });

function fakeTranslit(rulesOnly: readonly string[] = []) {
  vi.mocked(urduApi.translitSuggest).mockImplementation(({ text, direction }) =>
    Promise.resolve(
      suggestionsFor(
        {
          toUrdu: {
            kot: ['کوٹ'],
            radha: ['رادھا'],
            kishan: ['کشن'],
            sahiwal: ['ساہیوال'],
            zaheer: ['ظہیر'],
          },
          toEnglish: { کوٹ: 'Kot', رادھا: 'Radha', کشن: 'Kishan', ساہیوال: 'Sahiwal' },
          rulesOnly,
        },
        text,
        direction,
      ),
    ),
  );
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
}

/** The text of a grid cell, by the row's id and the column's id. */
function cell(rowId: number, colId: string): string {
  const row = document.querySelector(`[role="row"][row-id="${String(rowId)}"]`);
  return visible(row?.querySelector(`[col-id="${colId}"]`)?.textContent);
}

/** The element of a grid cell, for a click. */
function cellElement(rowId: number, colId: string): HTMLElement {
  const found = document.querySelector<HTMLElement>(
    `.ag-row[row-id="${String(rowId)}"] [col-id="${colId}"]`,
  );
  if (found === null) throw new Error(`no cell ${String(rowId)}:${colId}`);
  return found;
}

const rowIds = () =>
  Array.from(document.querySelectorAll('.ag-row[row-id]')).map((row) => row.getAttribute('row-id'));

/** Matches an accessible name or a label without Fluent's isolation marks. */
const named = (text: string) => (name: string) => visible(name) === text;

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeTranslit();
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the انڈکس menu (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeMasters();
  });

  it('lists the accounts, the chart, the rules and the nine lists in order, and ↓ then Enter opens one', async () => {
    const { router } = await renderApp('/index');
    const heading = await screen.findByRole('heading', {
      level: 1,
      name: message(language, 'masters-menu-title'),
    });
    expect(heading).toBeInTheDocument();
    const links = screen.getAllByRole('link');
    const shown = [
      { titleId: 'masters-accounts-title', helpId: 'masters-accounts-help' },
      { titleId: 'masters-chart-title', helpId: 'masters-chart-help' },
      { titleId: 'masters-rules-title', helpId: 'masters-rules-help' },
      ...MASTER_CONFIGS,
    ];
    expect(links).toHaveLength(shown.length);
    links.forEach((link, index) => {
      const entry = shown[index];
      expect(visible(link.textContent)).toContain(message(language, entry?.titleId ?? ''));
      expect(visible(link.textContent)).toContain(message(language, entry?.helpId ?? ''));
    });
    // The first list has focus when the menu opens.
    expect(links[0]).toHaveFocus();
    const user = userEvent.setup();
    await user.keyboard('{ArrowDown}{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/index/chart');
    });
  });

  it('an address that is not one of the nine lists says so, with the way back', async () => {
    const { router } = await renderApp('/index/nothing');
    expect(
      await screen.findByRole('heading', { name: message(language, 'masters-unknown-title') }),
    ).toBeInTheDocument();
    await userEvent
      .setup()
      .click(screen.getByRole('button', { name: named(message(language, 'masters-menu-back')) }));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/index');
    });
  });
});

describe.each(LANGUAGES)('a master list (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('shows the rows with the name in the app language, the other script, and the state', async () => {
    fakeMasters({ city: [KOT, LAHORE, OLD] });
    await renderApp('/index/cities');
    await screen.findByRole('heading', { level: 1, name: message(language, 'masters-city-title') });
    await waitFor(() => {
      expect(rowIds()).toEqual(['1', '2']);
    });
    expect(cell(1, 'code')).toBe('0001');
    expect(cell(1, 'name')).toBe(language === 'ur' ? KOT.names?.name_ur : KOT.names?.name_en);
    expect(cell(1, 'other_name')).toBe(language === 'ur' ? KOT.names?.name_en : KOT.names?.name_ur);
    expect(cell(1, 'status')).toBe(message(language, 'masters-status-active'));
    // A header per column, from Fluent.
    expect(screen.getByText(message(language, 'masters-col-other-name'))).toBeInTheDocument();
  });

  it('"show the inactive ones too" adds them, marked inactive in words as well as dimmed', async () => {
    fakeMasters({ city: [KOT, LAHORE, OLD] });
    await renderApp('/index/cities');
    await waitFor(() => {
      expect(rowIds()).toEqual(['1', '2']);
    });
    await userEvent
      .setup()
      .click(screen.getByRole('checkbox', { name: message(language, 'masters-show-inactive') }));
    await waitFor(() => {
      expect(rowIds()).toEqual(['1', '2', '3']);
    });
    expect(cell(3, 'status')).toBe(message(language, 'masters-status-inactive'));
    expect(document.querySelector('[row-id="3"]')).toHaveClass('row-inactive');
  });

  it('the search keeps the matches of the backend search, in both scripts', async () => {
    fakeMasters({ city: [KOT, LAHORE] });
    await renderApp('/index/cities');
    const box = await screen.findByLabelText(message(language, 'masters-filter-label'));
    expect(box).toHaveFocus();
    const user = userEvent.setup();
    await user.keyboard('lah');
    await waitFor(() => {
      expect(rowIds()).toEqual(['2']);
    });
    expect(searchApi.searchMasters).toHaveBeenLastCalledWith({
      scope: 'city',
      text: 'lah',
      limit: 500,
    });
    await user.clear(box);
    await user.keyboard('زzz');
    expect(
      await screen.findByRole('heading', { name: message(language, 'masters-no-match-title') }),
    ).toBeInTheDocument();
    await user.click(
      screen.getByRole('button', { name: message(language, 'masters-no-match-clear') }),
    );
    await waitFor(() => {
      expect(rowIds()).toEqual(['1', '2']);
    });
  });

  it('150 cities come page by page: 100 at once with the count of the backend, Show more loads the rest', async () => {
    const cities = Array.from({ length: 150 }, (_, index) =>
      record(
        'city',
        index + 1,
        String(index + 1).padStart(4, '0'),
        `شہر ${String(index + 1)}`,
        `City ${String(index + 1)}`,
      ),
    );
    fakeMasters({ city: cities });
    await renderApp('/index/cities');
    await screen.findByRole('heading', { level: 1, name: message(language, 'masters-city-title') });
    const showing = (shown: number) =>
      message(language, 'common-list-count', { shown: String(shown), total: '150' });
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(showing(100));
    });
    expect(mastersApi.mastersList).toHaveBeenLastCalledWith('city', true, 0, 100);
    await userEvent.setup().click(
      await screen.findByRole('button', {
        name: (name) => visible(name) === message(language, 'common-list-more'),
      }),
    );
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(showing(150));
    });
    expect(mastersApi.mastersList).toHaveBeenLastCalledWith('city', true, 100, 100);
  });

  it('the row last opened is on a later page: the pages are read until it is loaded, so the list comes back to it', async () => {
    const cities = Array.from({ length: 150 }, (_, index) =>
      record(
        'city',
        index + 1,
        String(index + 1).padStart(4, '0'),
        `شہر ${String(index + 1)}`,
        `City ${String(index + 1)}`,
      ),
    );
    fakeMasters({ city: cities });
    useMasterFocus.setState({ rows: { city: 140 } });
    await renderApp('/index/cities');
    await waitFor(
      () => {
        expect(mastersApi.mastersList).toHaveBeenLastCalledWith('city', true, 100, 100);
      },
      { timeout: 15_000 },
    );
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(
        message(language, 'common-list-count', { shown: '150', total: '150' }),
      );
    });
  });

  it('a search finds a city that is on a later page: the rest of the pages load for it', async () => {
    const cities = Array.from({ length: 150 }, (_, index) =>
      record(
        'city',
        index + 1,
        String(index + 1).padStart(4, '0'),
        `شہر ${String(index + 1)}`,
        `City ${String(index + 1)}`,
      ),
    );
    cities[139] = record('city', 140, '0140', 'ظفروال', 'Zafarwal');
    // The list remembers the row it last focused; a run before this one may have left one.
    useMasterFocus.setState({ rows: {} });
    fakeMasters({ city: cities });
    await renderApp('/index/cities');
    const box = await screen.findByLabelText(message(language, 'masters-filter-label'));
    await userEvent.setup().type(box, 'zafar');
    await waitFor(
      () => {
        expect(rowIds()).toEqual(['140']);
      },
      { timeout: 15_000 },
    );
    // Both pages were read: the match was on the second.
    expect(mastersApi.mastersList).toHaveBeenLastCalledWith('city', true, 100, 100);
  });

  it('with nothing in it, says why and offers the key for a new one', async () => {
    fakeMasters();
    const { router } = await renderApp('/index/cities');
    expect(
      await screen.findByRole('heading', { name: named(message(language, 'masters-city-empty')) }),
    ).toBeInTheDocument();
    expect(visible(document.body.textContent)).toContain(
      message(language, 'masters-empty-help', { key: 'F2' }),
    );
    press('F2');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/index/cities/new');
    });
  });

  it('a failure shows the message with a reference and a retry that works', async () => {
    fakeMasters({ city: [KOT] });
    vi.mocked(mastersApi.mastersList).mockRejectedValueOnce(fail({ code: 'ERR_DB_BUSY' }));
    await renderApp('/index/cities');
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'masters-load-failed'));
    expect(visible(alert.textContent)).toContain(message(language, 'err-db-busy'));
    expect(alert.textContent).not.toContain('ERR_');
    await userEvent
      .setup()
      .click(within(alert).getByRole('button', { name: message(language, 'common-retry') }));
    await waitFor(() => {
      expect(rowIds()).toEqual(['1']);
    });
  });

  it('Enter on a row opens it, and Esc comes back to the list with that row focused', async () => {
    fakeMasters({ city: [KOT, LAHORE] });
    const { router } = await renderApp('/index/cities');
    await waitFor(() => {
      expect(rowIds()).toEqual(['1', '2']);
    });
    const user = userEvent.setup();
    await user.click(cellElement(2, 'code'));
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/index/cities/2');
    });
    expect(
      await screen.findByRole('heading', {
        level: 1,
        name: named(
          message(language, 'masters-edit-title', {
            name: language === 'ur' ? 'لاہور' : 'Lahore',
          }),
        ),
      }),
    ).toBeInTheDocument();
    press('Escape');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/index/cities');
    });
    await waitFor(() => {
      expect(document.querySelector('[row-id="2"] .ag-cell-focus')).not.toBeNull();
    });
  });
});

describe.each(LANGUAGES)('adding and changing a master (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('a new city by keyboard only: F2, the name, Enter, Enter; the next code is already there', async () => {
    const store = fakeMasters({ city: [KOT, LAHORE] });
    const { router } = await renderApp('/index/cities');
    await waitFor(() => {
      expect(rowIds()).toEqual(['1', '2']);
    });
    press('F2');
    const code = await screen.findByLabelText<HTMLInputElement>(
      message(language, 'masters-field-code'),
    );
    expect(code.value).toBe('0003');
    expect(code).toHaveFocus();
    const user = userEvent.setup();
    if (language === 'ur') {
      // Code, then the Urdu name typed in English letters, then the English name that filled itself.
      await user.keyboard('{Enter}kot radha kishan{Enter}');
    } else {
      await user.keyboard('{Enter}Kot Radha Kishan{Enter}');
    }
    // Enter on the last field saves.
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(mastersApi.mastersSave).toHaveBeenCalledTimes(1);
    });
    const [kind, draft, confirmed] = vi.mocked(mastersApi.mastersSave).mock.calls[0] ?? [];
    expect(kind).toBe('city');
    expect(draft).toMatchObject({
      id: null,
      code: '0003',
      names: { urdu: 'کوٹ رادھا کشن', english: 'Kot Radha Kishan' },
    });
    expect(confirmed).toEqual([]);
    expect(store.rows.city).toHaveLength(3);
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/index/cities');
    });
    expect(await screen.findByText(message(language, 'masters-saved'))).toBeInTheDocument();
  });

  it('changing a city sends its id and the version that was read', async () => {
    fakeMasters({ city: [KOT, LAHORE] });
    await renderApp('/index/cities/2');
    const name = await screen.findByLabelText<HTMLInputElement>(
      message(language, language === 'ur' ? 'masters-field-name-ur' : 'masters-field-name-en'),
    );
    const user = userEvent.setup();
    await user.click(name);
    await user.keyboard('{End} 2');
    press('F10');
    await waitFor(() => {
      expect(mastersApi.mastersSave).toHaveBeenCalledTimes(1);
    });
    const [, draft] = vi.mocked(mastersApi.mastersSave).mock.calls[0] ?? [];
    expect(draft).toMatchObject({ id: 2, version: 1, code: '0002' });
  });

  it('a refusal at the code sits under the code field, and the form stays as typed', async () => {
    fakeMasters({ city: [KOT] });
    vi.mocked(mastersApi.mastersSave).mockRejectedValueOnce(
      fail({ code: 'ERR_CODE_INVALID', field: 'code' }),
    );
    await renderApp('/index/cities/new');
    const code = await screen.findByLabelText<HTMLInputElement>(
      message(language, 'masters-field-code'),
    );
    const user = userEvent.setup();
    await user.clear(code);
    await user.type(code, '77');
    press('F10');
    await waitFor(() => {
      expect(code).toHaveAttribute('aria-invalid', 'true');
    });
    expect(
      visible(document.getElementById(code.getAttribute('aria-describedby') ?? '')?.textContent),
    ).toBe(message(language, 'err-code-invalid'));
    expect(code.value).toBe('77');
  });

  it('leaving with unsaved changes asks first; leaving without saving drops them', async () => {
    fakeMasters({ city: [KOT] });
    const { router } = await renderApp('/index/cities/1');
    const name = await screen.findByLabelText<HTMLInputElement>(
      message(language, language === 'ur' ? 'masters-field-name-ur' : 'masters-field-name-en'),
    );
    const user = userEvent.setup();
    await user.click(name);
    await user.keyboard('{End}x');
    press('Escape');
    const prompt = await screen.findByRole('alertdialog');
    expect(visible(prompt.textContent)).toContain(message(language, 'layout-save-title'));
    expect(router.state.location.pathname).toBe('/index/cities/1');
    await user.click(
      within(prompt).getByRole('button', {
        name: new RegExp(message(language, 'layout-save-leave')),
      }),
    );
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/index/cities');
    });
    expect(mastersApi.mastersSave).not.toHaveBeenCalled();
  });

  it('making a city inactive asks first, says what happens, and nothing is deleted', async () => {
    const store = fakeMasters({ city: [KOT, LAHORE] });
    const { router } = await renderApp('/index/cities/2');
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole('button', { name: message(language, 'masters-make-inactive') }),
    );
    const box = await screen.findByRole('alertdialog');
    expect(visible(box.textContent)).toContain(
      message(language, 'masters-inactive-description', {
        name: language === 'ur' ? 'لاہور' : 'Lahore',
      }),
    );
    expect(mastersApi.mastersSetActive).not.toHaveBeenCalled();
    await user.click(
      within(box).getByRole('button', {
        name: new RegExp(message(language, 'masters-make-inactive')),
      }),
    );
    await waitFor(() => {
      expect(mastersApi.mastersSetActive).toHaveBeenCalledWith('city', 2, false);
    });
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/index/cities');
    });
    expect(store.rows.city).toHaveLength(2);
    await waitFor(() => {
      expect(rowIds()).toEqual(['1']);
    });
  });

  it('an inactive city says so, and "make active" brings it back', async () => {
    fakeMasters({ city: [KOT, OLD] });
    await renderApp('/index/cities/3');
    expect(await screen.findByText(message(language, 'masters-inactive-note'))).toBeInTheDocument();
    await userEvent
      .setup()
      .click(screen.getByRole('button', { name: message(language, 'masters-make-active') }));
    await waitFor(() => {
      expect(mastersApi.mastersSetActive).toHaveBeenCalledWith('city', 3, true);
    });
  });

  it('the form asks for the one master by its id, so a city far down a long list opens at once', async () => {
    const cities = Array.from({ length: 150 }, (_, index) =>
      record(
        'city',
        index + 1,
        String(index + 1).padStart(4, '0'),
        `شہر ${String(index + 1)}`,
        `City ${String(index + 1)}`,
      ),
    );
    fakeMasters({ city: cities });
    await renderApp('/index/cities/140');
    expect(await screen.findByDisplayValue('City 140')).toBeInTheDocument();
    expect(mastersApi.mastersGetOne).toHaveBeenCalledWith('city', 140);
    // No page of the list was read to find it.
    expect(mastersApi.mastersList).not.toHaveBeenCalled();
  });

  it('a master that is gone says so instead of showing an empty form', async () => {
    fakeMasters({ city: [KOT] });
    await renderApp('/index/cities/99');
    expect(
      await screen.findByRole('heading', { name: message(language, 'masters-not-found-title') }),
    ).toBeInTheDocument();
  });

  it('a language switch in the middle of an entry keeps every value', async () => {
    fakeMasters({ city: [KOT] });
    await renderApp('/index/cities/new');
    const urdu = await screen.findByLabelText<HTMLInputElement>(
      message(language, 'masters-field-name-ur'),
    );
    const user = userEvent.setup();
    await user.click(urdu);
    await user.keyboard('sahiwal ');
    await waitFor(() => {
      expect(urdu.value).toBe('ساہیوال ');
    });
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.getState().setLanguage(other);
    });
    expect(
      await screen.findByLabelText<HTMLInputElement>(message(other, 'masters-field-name-ur')),
    ).toHaveValue('ساہیوال ');
    expect(
      screen.getByLabelText<HTMLInputElement>(message(other, 'masters-field-name-en')).value,
    ).toBe('Sahiwal');
    expect(
      screen.getByLabelText<HTMLInputElement>(message(other, 'masters-field-code')).value,
    ).toBe('0002');
  });
});

describe.each(LANGUAGES)('the spelling check on a master (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeTranslit(['zaheer']);
  });

  it('a name with a rules-only word asks to confirm; confirming saves and sends the word', async () => {
    fakeMasters({ city: [] });
    await renderApp('/index/cities/new');
    const urdu = await screen.findByLabelText<HTMLInputElement>(
      message(language, 'masters-field-name-ur'),
    );
    const user = userEvent.setup();
    await user.click(urdu);
    await user.keyboard('zaheer ');
    await waitFor(() => {
      expect(screen.getByRole('status').textContent).toContain('ظہیر');
    });
    press('F10');
    const box = await screen.findByRole('alertdialog');
    expect(box.querySelector('mark')?.textContent).toBe('ظہیر');
    expect(mastersApi.mastersSave).not.toHaveBeenCalled();
    await user.click(
      within(box).getByRole('button', {
        name: new RegExp(message(language, 'translit-spelling-confirm')),
      }),
    );
    await waitFor(() => {
      expect(mastersApi.mastersSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(mastersApi.mastersSave).mock.calls[0]?.[2]).toEqual(['ظہیر']);
  });

  it('a narration is free text: no spelling check, no dialog', async () => {
    fakeMasters();
    await renderApp('/index/narrations/new');
    const urdu = await screen.findByLabelText<HTMLInputElement>(
      message(language, 'masters-narration-text-ur'),
    );
    const user = userEvent.setup();
    await user.click(urdu);
    await user.keyboard('zaheer ');
    await waitFor(() => {
      expect(urdu.value).toBe('ظہیر ');
    });
    expect(
      screen.queryByText(message(language, 'translit-spelling-check')),
    ).not.toBeInTheDocument();
    press('F10');
    await waitFor(() => {
      expect(mastersApi.mastersSave).toHaveBeenCalledTimes(1);
    });
    expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
  });
});

describe.each(LANGUAGES)('masters with their own fields (%s)', (language) => {
  const BOX = record('packing', 11, '0011', 'پیٹی', 'Box', { text: { approx_kg: '20' } });
  const CRATE = record('packing', 12, '0012', 'کریٹ', 'Crate', { text: { approx_kg: '15' } });

  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('an item has a group, a default packing found with the picker, and how it is sold', async () => {
    const store = fakeMasters({ packing: [BOX, CRATE] });
    await renderApp('/index/items/new');
    const user = userEvent.setup();
    const nameLabel = message(
      language,
      language === 'ur' ? 'masters-field-name-ur' : 'masters-field-name-en',
    );
    await user.click(await screen.findByLabelText(nameLabel));
    await user.keyboard(language === 'ur' ? 'kot ' : 'Kot ');
    // The picker searches, in both scripts, and never converts English letters to Urdu.
    const packing = screen.getByRole<HTMLInputElement>('combobox', {
      name: named(message(language, 'masters-item-packing')),
    });
    await user.click(packing);
    await user.keyboard('cra');
    await waitFor(() => {
      expect(searchApi.searchMasters).toHaveBeenCalledWith(
        expect.objectContaining({ scope: 'packing', text: 'cra' }),
      );
    });
    await waitFor(() => {
      expect(screen.getAllByRole('option')).toHaveLength(1);
    });
    await user.click(screen.getByRole('option', { name: /Crate|کریٹ/ }));
    expect(urduApi.translitSuggest).not.toHaveBeenCalledWith(
      expect.objectContaining({ text: 'cra' }),
    );
    press('F10');
    await waitFor(() => {
      expect(mastersApi.mastersSave).toHaveBeenCalledTimes(1);
    });
    const [kind, draft] = vi.mocked(mastersApi.mastersSave).mock.calls[0] ?? [];
    expect(kind).toBe('item');
    expect(draft?.text).toMatchObject({ item_group: 'fruit', sold_by: 'unit' });
    expect(draft?.refs['default_packing']).toMatchObject({ id: 12, code: '0012' });
    expect(store.rows.item).toHaveLength(1);
  });

  it('a packing keeps its approximate weight as typed text', async () => {
    fakeMasters({ packing: [BOX] });
    await renderApp('/index/packings/11');
    const weight = await screen.findByLabelText<HTMLInputElement>(
      message(language, 'masters-packing-weight'),
    );
    expect(weight.value).toBe('20');
    const user = userEvent.setup();
    await user.click(weight);
    await user.keyboard('{Control>}a{/Control}25');
    press('F10');
    await waitFor(() => {
      expect(mastersApi.mastersSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(mastersApi.mastersSave).mock.calls[0]?.[1].text).toMatchObject({
      approx_kg: '25',
    });
  });

  it('an expense head shows the account the backend made for it, read-only', async () => {
    fakeMasters({
      expense_head: [
        record('expense_head', 21, '0001', 'بجلی', 'Electricity', {
          shown: { account_code: '05010001' },
        }),
      ],
    });
    await renderApp('/index/expense-heads');
    await waitFor(() => {
      expect(cell(21, 'account_code')).toBe('05010001');
    });
    await userEvent.setup().click(cellElement(21, 'code'));
    await userEvent.setup().keyboard('{Enter}');
    const account = await screen.findByLabelText<HTMLInputElement>(
      message(language, 'masters-account-code'),
    );
    expect(account.value).toBe('05010001');
    expect(account).toHaveAttribute('readonly');
  });

  it('a staff member shows the money-lent account link, empty until he is first lent money', async () => {
    fakeMasters({
      staff: [
        record('staff', 31, '', 'اسلم', 'Aslam', {
          text: { job: 'munshi', phone: '03001234567' },
        }),
      ],
    });
    await renderApp('/index/staff/31');
    const link = await screen.findByLabelText<HTMLInputElement>(
      named(message(language, 'masters-staff-account')),
    );
    expect(link.value).toBe('');
    expect(link).toHaveAttribute('readonly');
    expect(visible(document.body.textContent)).toContain(
      message(language, 'masters-staff-account-hint'),
    );
    // A master with no code of its own has no code field.
    expect(
      screen.queryByLabelText(message(language, 'masters-field-code')),
    ).not.toBeInTheDocument();
  });

  it('a mark has an owner seller, found with the account picker and sent back as chosen', async () => {
    const RAHIM = account(31, 'seller', '030110001', 'رحیم', 'Rahim');
    fakeAccountSearch([RAHIM]);
    fakeMasters();
    await renderApp('/index/markas/new');
    const user = userEvent.setup();
    await user.click(await screen.findByLabelText(named(message(language, 'masters-marka-text'))));
    await user.keyboard('AR');
    const owner = screen.getByRole<HTMLInputElement>('combobox', {
      name: named(message(language, 'masters-marka-owner')),
    });
    await user.click(owner);
    await user.keyboard('rah');
    await waitFor(() => {
      expect(screen.getAllByRole('option')).toHaveLength(1);
    });
    await user.keyboard('{Enter}');
    expect(owner.value).toBe(language === 'ur' ? 'رحیم' : 'Rahim');
    press('F10');
    await waitFor(() => {
      expect(mastersApi.mastersSave).toHaveBeenCalledTimes(1);
    });
    const [kind, draft] = vi.mocked(mastersApi.mastersSave).mock.calls[0] ?? [];
    expect(kind).toBe('marka');
    expect(draft?.text).toMatchObject({ mark_text: 'AR' });
    expect(draft?.refs['owner']).toMatchObject({ id: 31, code: '030110001' });
  });

  it('a narration is offered in the voucher types that are ticked, and in all when none is', async () => {
    fakeMasters();
    await renderApp('/index/narrations/new');
    const user = userEvent.setup();
    await user.click(await screen.findByLabelText(message(language, 'masters-narration-text-ur')));
    await user.keyboard('x');
    // Nothing is ticked at first: the hint says that means every voucher.
    expect(visible(document.body.textContent)).toContain(
      message(language, 'masters-narration-used-in-hint'),
    );
    await user.click(
      screen.getByRole('checkbox', { name: named(message(language, 'masters-used-in-receipt')) }),
    );
    await user.click(
      screen.getByRole('checkbox', {
        name: named(message(language, 'masters-used-in-personal_return')),
      }),
    );
    press('F10');
    await waitFor(() => {
      expect(mastersApi.mastersSave).toHaveBeenCalledTimes(1);
    });
    const [kind, draft] = vi.mocked(mastersApi.mastersSave).mock.calls[0] ?? [];
    expect(kind).toBe('narration');
    expect([...(draft?.lists['used_in'] ?? [])].sort()).toEqual(['personal_return', 'receipt']);
  });

  it('a saved narration shows its voucher types ticked, and untick sends them out of the list', async () => {
    fakeMasters({
      narration: [
        record('narration', 51, '', 'بجلی کا بل', 'Electricity bill', {
          lists: { used_in: ['shop_expense', 'payment'] },
        }),
      ],
    });
    await renderApp('/index/narrations/51');
    const user = userEvent.setup();
    const shop = await screen.findByRole<HTMLInputElement>('checkbox', {
      name: named(message(language, 'masters-used-in-shop_expense')),
    });
    expect(shop.checked).toBe(true);
    expect(
      screen.getByRole<HTMLInputElement>('checkbox', {
        name: named(message(language, 'masters-used-in-receipt')),
      }).checked,
    ).toBe(false);
    await user.click(shop);
    press('F10');
    await waitFor(() => {
      expect(mastersApi.mastersSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(mastersApi.mastersSave).mock.calls[0]?.[1].lists['used_in']).toEqual([
      'payment',
    ]);
  });

  it('a mark keeps its text exactly as typed, with no names', async () => {
    fakeMasters({
      marka: [record('marka', 41, '0001', '', '', { names: null, text: { mark_text: 'AR' } })],
    });
    await renderApp('/index/markas');
    await waitFor(() => {
      expect(cell(41, 'mark_text')).toBe('AR');
    });
    expect(screen.queryByText(message(language, 'masters-col-name'))).not.toBeInTheDocument();
  });
});
