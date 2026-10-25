// Printing the collection list from its screen (plan 17e F2, PRD FR-15.6, FR-7.36, FR-7.57): the
// Print button and F9 on the collection screen, the print box with the day and the filters of the
// screen, keyboard only; the language of the print apart from the app, printer or PDF, copies; the
// preview with the backend's REAL print data (tests/fixtures, copied, never edited): loading, the
// list, nothing to print, and an error with a retry; the refusal on a computer that is not Windows
// told in words; and a language switch with the box open. In the Urdu and the English app.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as reportsApi from '../../src/features/reports/api';
import * as vouchersApi from '../../src/features/vouchers/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
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
import { fakeMasters, record } from '../helpers/masters';
import { EMPTY_PAIRS } from '../helpers/paged';
import { fakePrinting } from '../helpers/print';
import { message, visible } from '../helpers/render';
import * as real from '../helpers/tag-real';
import { AKRAM, ARSHAD, collectionList, fakeVouchers } from '../helpers/vouchers';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/features/pukhta_khata/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const named = (text: string) => (name: string) => visible(name) === text;
const pageText = () => visible(document.body.textContent);
function must<T>(found: T | null | undefined): T {
  if (found === null || found === undefined) throw new Error('the element is not on the page');
  return found;
}

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  fakeAccounts([AKRAM, ARSHAD]);
  fakeAccountSearch([AKRAM, ARSHAD]);
  fakeMasters({
    staff: [record('staff', 81, '0001', 'اسلم', 'Aslam')],
    city: [record('city', 91, '0001', 'کوٹ رادھا کشن', 'Kot Radha Kishan')],
  });
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('printing the collection list from its screen (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeVouchers();
    fakePrinting();
    vi.mocked(vouchersApi.assistCollectionList).mockResolvedValue(collectionList());
    vi.mocked(reportsApi.reportsCollectionPrintData).mockImplementation((input) =>
      Promise.resolve(input.language === 'ur' ? real.COLLECTION_UR : real.COLLECTION_EN),
    );
    vi.mocked(reportsApi.reportsPrintCollection).mockResolvedValue({
      printed: ['collection_list'],
      pdf_paths: null,
    });
  });

  async function open() {
    const app = await renderApp('/bikri/collection');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'vouchers-collection-title')),
    });
    await waitFor(() => {
      expect(document.querySelectorAll('.ag-row[row-index]').length).toBeGreaterThan(0);
    });
    return { ...app, user: userEvent.setup() };
  }
  const theBox = () => screen.findByRole('dialog');
  const boxButton = (box: HTMLElement, id: string) =>
    within(box).getByRole('button', { name: named(message(language, id)) });
  const printButton = () =>
    screen.getAllByRole('button', {
      name: (name) => visible(name).startsWith(message(language, 'vouchers-print')),
    })[0];

  it('the screen has a Print button that shows its key, and it opens the box with the day of the screen', async () => {
    const { user } = await open();
    const button = must(printButton());
    expect(visible(button.textContent)).toContain('F9');
    await user.click(button);
    const box = await theBox();
    expect(visible(box.textContent)).toContain(
      message(language, 'vouchers-collection-print-title'),
    );
    // The day of the screen, as typed there.
    expect(visible(box.textContent)).toContain('10-10-2026');
  });

  it('keyboard only: F9 opens the box, Tab reaches Print, Enter prints the list of the screen, Esc closes', async () => {
    const { user } = await open();
    press('F9');
    const box = await theBox();
    const print = boxButton(box, 'print-box-print');
    for (let step = 0; step < 30 && document.activeElement !== print; step += 1) await user.tab();
    expect(document.activeElement).toBe(print);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(reportsApi.reportsPrintCollection).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(reportsApi.reportsPrintCollection).mock.calls[0]?.[0]).toEqual({
      date: '10-10-2026',
      collector_staff_id: null,
      city_id: null,
      language,
      copies: null,
      to: 'printer',
    });
    await waitFor(() => {
      // "Sent to the printer: <the name of the list>" (the words of the print box, not a code).
      expect(visible(box.textContent)).toContain(message(language, 'print-doc-collection-list'));
      expect(visible(box.textContent)).toContain(
        message(language, 'print-box-done-printed', {
          docs: message(language, 'print-doc-collection-list'),
        }),
      );
    });
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
  });

  it('prints the day and the filters of the screen: the collector and the city he picked', async () => {
    const { user } = await open();
    for (const [id, typed, shown] of [
      ['vouchers-field-collector', 'aslam', language === 'ur' ? 'اسلم' : 'Aslam'],
      ['accounts-field-city', 'kot', language === 'ur' ? 'کوٹ رادھا کشن' : 'Kot Radha Kishan'],
    ] as const) {
      const field = screen.getByRole('combobox', { name: named(message(language, id)) });
      await user.click(field);
      await user.keyboard(typed);
      await user.click(
        await screen.findByRole('option', { name: (name) => visible(name).includes(shown) }),
      );
    }
    press('F9');
    const box = await theBox();
    await user.click(boxButton(box, 'print-box-print'));
    await waitFor(() => {
      expect(reportsApi.reportsPrintCollection).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(reportsApi.reportsPrintCollection).mock.calls[0]?.[0]).toMatchObject({
      date: '10-10-2026',
      collector_staff_id: 81,
      city_id: 91,
    });
  });

  it("the preview shows the backend's real list: the buyers with the tag, the dues, the total of this list", async () => {
    const { user } = await open();
    press('F9');
    const box = await theBox();
    await user.click(boxButton(box, 'print-box-preview'));
    const sheet = await waitFor(() => must(document.querySelector<HTMLElement>('.print-doc')));
    // The request is the one the backend's own test sent for the same day.
    expect(reportsApi.reportsCollectionPrintData).toHaveBeenCalledWith({
      ...real.COLLECTION_UR_REQUEST,
      language,
    });
    const text = visible(sheet.textContent);
    expect(text).toContain(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    expect(text).toContain('2,880');
    expect(text).toContain('18,684');
    expect(text).toContain('21,564');
    expect(text).toContain(message(language, 'print-collection-total'));
    expect(text).toContain(message(language, 'print-collection-list-title'));
    // Back to the form with the words of the box.
    await user.click(boxButton(box, 'common-back'));
    expect(
      await screen.findByRole('button', { name: named(message(language, 'print-box-print')) }),
    ).toBeInTheDocument();
  });

  it('the language of the print can differ from the app, PDF sends `pdf` and lists the file made', async () => {
    const other: Language = language === 'ur' ? 'en' : 'ur';
    vi.mocked(reportsApi.reportsPrintCollection).mockResolvedValue({
      printed: ['collection_list'],
      pdf_paths: ['C:\\Mandi\\collection-list-20261010-' + other + '.pdf'],
    });
    const { user } = await open();
    press('F9');
    const box = await theBox();
    await user.click(
      within(box).getByRole('button', {
        name: (name) => visible(name) === message(language, `common-language-${other}`),
      }),
    );
    await user.click(boxButton(box, 'print-box-to-pdf'));
    await user.click(boxButton(box, 'print-box-save-pdf'));
    await waitFor(() => {
      expect(reportsApi.reportsPrintCollection).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(reportsApi.reportsPrintCollection).mock.calls[0]?.[0]).toMatchObject({
      language: other,
      to: 'pdf',
    });
    await waitFor(() => {
      expect(visible(box.textContent)).toContain(`collection-list-20261010-${other}.pdf`);
    });
    // The app did not change language.
    expect(document.documentElement.lang).toBe(language);
  });

  it('on a computer that is not Windows the refusal is told in words, and the preview still shows the list', async () => {
    vi.mocked(reportsApi.reportsPrintCollection).mockRejectedValue(
      fail({ code: 'ERR_PRINT_WINDOWS_ONLY' }),
    );
    const { user } = await open();
    press('F9');
    const box = await theBox();
    await user.click(boxButton(box, 'print-box-print'));
    const alert = await within(box).findByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'err-print-windows-only'));
    expect(visible(alert.textContent)).not.toContain('ERR_');
    await user.click(boxButton(box, 'print-box-preview'));
    await waitFor(() => {
      expect(document.querySelector('.print-doc')).not.toBeNull();
    });
  });

  it('four states of the preview: loading, the list, nothing to print, an error with a retry', async () => {
    const { user } = await open();
    press('F9');
    const box = await theBox();
    // Loading: the answer has not come yet.
    let release: ((data: typeof real.COLLECTION_UR) => void) | undefined;
    vi.mocked(reportsApi.reportsCollectionPrintData).mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          release = resolve;
        }),
    );
    await user.click(boxButton(box, 'print-box-preview'));
    expect(await within(box).findByRole('status')).toBeInTheDocument();
    release?.(language === 'ur' ? real.COLLECTION_UR : real.COLLECTION_EN);
    // The list.
    await waitFor(() => {
      expect(document.querySelector('.print-doc')).not.toBeNull();
    });
    await user.click(boxButton(box, 'common-back'));
    // Nothing to print: an empty day has its own words and no sheet.
    vi.mocked(reportsApi.reportsCollectionPrintData).mockResolvedValue(real.COLLECTION_EMPTY);
    await user.click(boxButton(box, 'print-box-preview'));
    expect(
      await within(box).findByText(message(language, 'print-preview-empty')),
    ).toBeInTheDocument();
    expect(document.querySelector('.print-doc')).toBeNull();
    await user.click(boxButton(box, 'common-back'));
    // An error says so in words and has a retry that reads again.
    vi.mocked(reportsApi.reportsCollectionPrintData)
      .mockRejectedValueOnce(fail({ code: 'ERR_NOT_FOUND', field: 'collector_staff_id' }))
      .mockResolvedValue(language === 'ur' ? real.COLLECTION_UR : real.COLLECTION_EN);
    await user.click(boxButton(box, 'print-box-preview'));
    expect(
      await within(box).findByText(message(language, 'print-preview-failed')),
    ).toBeInTheDocument();
    await user.click(boxButton(box, 'common-retry'));
    await waitFor(() => {
      expect(document.querySelector('.print-doc')).not.toBeNull();
    });
  });

  it('a language switch with the box open changes its words and keeps what was typed (the copies, the choice)', async () => {
    const { user } = await open();
    press('F9');
    const box = await theBox();
    const copies = within(box).getByLabelText<HTMLInputElement>(
      named(message(language, 'print-box-copies')),
    );
    await user.type(copies, '3');
    await user.click(boxButton(box, 'print-box-to-pdf'));
    const other: Language = language === 'ur' ? 'en' : 'ur';
    press('KeyL', { altKey: true });
    await waitFor(() => {
      expect(visible(box.textContent)).toContain(message(other, 'vouchers-collection-print-title'));
    });
    expect(
      within(box).getByLabelText<HTMLInputElement>(named(message(other, 'print-box-copies'))).value,
    ).toBe('3');
    // The choice of PDF is still made: the main button says "Save PDF" in the new language.
    expect(boxButton2(box, other, 'print-box-save-pdf')).toBeInTheDocument();
    expect(pageText()).toContain('10-10-2026');
  });
});

function boxButton2(box: HTMLElement, language: Language, id: string) {
  return within(box).getByRole('button', { name: named(message(language, id)) });
}
