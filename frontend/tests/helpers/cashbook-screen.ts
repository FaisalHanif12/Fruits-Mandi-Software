// Test helper for the screens of the روکڑ area (plan 17): everything a test of the full app needs
// besides the `vi.mock` lines, which the test file itself must have (they are hoisted there):
//
//   vi.mock('../../src/app/api'); vi.mock('../../src/features/auth/api');
//   vi.mock('../../src/features/settings/api'); vi.mock('../../src/features/cashbook/api');
//   vi.mock('../../src/features/vouchers/api'); vi.mock('../../src/features/partners/api');
//   vi.mock('../../src/features/pukhta_khata/api'); vi.mock('../../src/features/reports/api');
//   vi.mock('../../src/features/masters/api'); vi.mock('../../src/shared/accounts/api');
//   vi.mock('../../src/shared/search/api'); vi.mock('../../src/shared/urdu/api');

import { screen, waitFor } from '@testing-library/react';
import type userEvent from '@testing-library/user-event';
import { expect, vi } from 'vitest';
import * as pukhtaApi from '../../src/features/pukhta_khata/api';
import * as reportsApi from '../../src/features/reports/api';
import type { Language } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeAccountSearch, fakeAccounts, type account } from './accounts';
import { cashbookTree, fakeCashbook, type FakeCashbook } from './cashbook';
import type { CashbookBooksOptions } from './cashbook-books';
import { healthyBackend, resetClientState, sessionUser, signedIn } from './app';
import { fakePartners, THREE_PARTNERS } from './partners';
import { fakeMasters, record } from './masters';
import { message, visible } from './render';

type AccountRow = ReturnType<typeof account>;

/** Sets every fake an app test of the روکڑ area needs and returns the books. */
export function prepareCashbook(
  language: Language,
  rows: AccountRow[] = [],
  options: CashbookBooksOptions = {},
): FakeCashbook {
  healthyBackend();
  resetClientState(language);
  signedIn(sessionUser({ language }));
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue([]);
  vi.mocked(pukhtaApi.searchPukhtaPeople).mockResolvedValue([]);
  vi.mocked(reportsApi.reportsListPrinters).mockResolvedValue([]);
  fakeAccounts(rows, cashbookTree());
  fakeAccountSearch(rows);
  fakePartners(THREE_PARTNERS);
  fakeMasters({ staff: [record('staff', 81, '0001', 'اسلم', 'Aslam')] });
  return fakeCashbook(options);
}

export const named = (text: string) => (name: string) => visible(name) === text;

/** A name that starts with this text: a button with its key chip, or a label with isolation marks. */
export const startsWith = (text: string) => (name: string) => visible(name).startsWith(text);

/** The input or button of a field, found by its label in the language. */
export const labelled = (language: Language, id: string) =>
  screen.findByLabelText<HTMLInputElement>(named(message(language, id)));

/** A drop-down or picker of a field, found by its label in the language. */
export const dropdown = (language: Language, id: string) =>
  screen.findByRole('combobox', { name: named(message(language, id)) });

/** The text of the whole page without isolation marks. */
export const pageText = () => visible(document.body.textContent);

/** Opens a drop-down (Radix Select) and takes the option with this text. */
export async function choose(
  user: ReturnType<typeof userEvent.setup>,
  language: Language,
  fieldId: string,
  optionText: string,
) {
  await user.click(await dropdown(language, fieldId));
  await user.click(await screen.findByRole('option', { name: named(optionText) }));
}

/** Types an amount in the amount field of the screen. */
export async function typeAmount(
  user: ReturnType<typeof userEvent.setup>,
  language: Language,
  text: string,
) {
  const amount = await labelled(language, 'cashbook-field-amount');
  await user.click(amount);
  await user.keyboard(text);
  await waitFor(() => {
    expect(amount.value).not.toBe('');
  });
}

/** Types in an account picker and takes the only match. */
export async function pickAccount(
  user: ReturnType<typeof userEvent.setup>,
  language: Language,
  fieldId: string,
  typed: string,
  shown: string,
) {
  await user.click(await dropdown(language, fieldId));
  await user.keyboard(typed);
  await waitFor(() => {
    const options = screen.getAllByRole('option');
    expect(options).toHaveLength(1);
    expect(visible(options[0]?.textContent)).toContain(shown);
  });
  await user.keyboard('{Enter}');
}
