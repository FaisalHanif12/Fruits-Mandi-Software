// Plan 78 with the backend's own files (tests/fixtures, written by its test plan78_fixtures): the card "where the
// capital is" draws the real answer of Events 1 to 5 (65,488 on both sides) and the real refusal while the partner
// section is locked. The commands and the values of the new refusals are in plan78-api-contract.test.ts.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as cashbookApi from '../../src/features/cashbook/api';
import { LANGUAGES } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import { prepareCashbook } from '../helpers/cashbook-screen';
import { leaveAppWindow, renderApp } from '../helpers/app';
import * as real from '../helpers/plan78-real';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/cashbook/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('where the capital is, with the real answers (%s)', (language) => {
  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);

  beforeEach(() => {
    vi.clearAllMocks();
    prepareCashbook(language, [], { cash: 58_484, cashRows: [] });
  });

  async function openCard() {
    await renderApp('/rokar');
    await screen.findByRole('heading', { level: 1, name: m('cashbook-title') });
    const user = userEvent.setup();
    await user.click(
      screen.getByRole('button', {
        name: (name) => visible(name).includes(m('cashbook-capital-title')),
      }),
    );
  }

  it('draws every line the backend sends, with both totals of 65,488', async () => {
    vi.mocked(cashbookApi.cashbookCapitalWhere).mockResolvedValue(real.CAPITAL_WHERE);
    await openCard();
    const where = await screen.findByRole('table', { name: m('cashbook-capital-where-title') });
    const whose = await screen.findByRole('table', { name: m('cashbook-capital-whose-title') });
    const whereText = visible(where.textContent);
    // Cash, what the buyers owe, minus what the shop owes the sellers and for charges.
    for (const figure of ['58,484', '21,564', '10,160', '4,400', '65,488']) {
      expect(whereText).toContain(figure);
    }
    // The four main places always show, also with nothing in them (advances with sellers is 0); every other place
    // shows only when it is not 0 (the backend sends them all).
    expect(whereText).toContain(m('cashbook-capital-where-seller-advances'));
    for (const id of ['separate-khatas', 'money-lent', 'gardens-held', 'sellers-owe-beyond']) {
      expect(whereText).not.toContain(m(`cashbook-capital-where-${id}`));
    }
    const whoseText = visible(whose.textContent);
    for (const figure of ['1,004', '746', '65,000', '65,488']) expect(whoseText).toContain(figure);
    const lastRow = (table: HTMLElement) => {
      const rows = within(table).getAllByRole('row');
      return visible(rows[rows.length - 1]?.textContent);
    };
    expect(lastRow(where)).toContain('65,488');
    expect(lastRow(whose)).toContain('65,488');
  });

  it('asks for the code in its own place while the partner section is locked', async () => {
    vi.mocked(cashbookApi.cashbookCapitalWhere).mockRejectedValue(
      new CommandError(real.CAPITAL_WHERE_LOCKED),
    );
    await openCard();
    expect(
      await screen.findByText((text) => visible(text) === m('partners-code-box-text')),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(screen.queryByRole('table', { name: m('cashbook-capital-where-title') })).toBeNull();
    });
  });
});
