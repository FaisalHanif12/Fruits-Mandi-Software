// The روکڑ بک with Umar's own books (plan 96 T2, PRD FR-16.14, FR-19.11), in the shape of plan 99. The answer is
// what the real backend sent for a copy of his books, 29-10-2026 to 27-11-2026: 23 lines, so the list is a long one. A list of more
// than ten rows is drawn in the 'fill' height mode (the book now has its own as-tall-as-its-rows height, but every
// other long list still uses it, see list-grid-fill-class.test.tsx), and its box once lost the AG Grid theme class (a missing
// space joined `ag-theme-quartz` and `list-grid-fill` into one unknown word), so the cards showed and the table
// was an empty strip. The dev page has four lines and never reached that mode. In the Urdu and the English app.

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as cashbookApi from '../../src/features/cashbook/api';
import { LANGUAGES } from '../../src/i18n';
import { GRID_THEME_CLASS } from '../../src/shared/grid/listGridClass';
import { named, pageText, prepareCashbook } from '../helpers/cashbook-screen';
import { CASHBOOK_UMAR_REAL } from '../helpers/cashbook-real';
import { leaveAppWindow, renderApp } from '../helpers/app';
import { rangeText } from '../helpers/pager';
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

beforeEach(() => {
  vi.clearAllMocks();
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)("the روکڑ بک with Umar's real books (%s)", (language) => {
  async function open() {
    prepareCashbook(language, [], { cash: 1_057_840 });
    vi.mocked(cashbookApi.cashbookGet).mockResolvedValue(CASHBOOK_UMAR_REAL);
    await renderApp('/rokar');
    await screen.findByRole('heading', { level: 1, name: message(language, 'cashbook-title') });
    return screen.findByRole('group', { name: named(message(language, 'cashbook-list-label')) });
  }

  it('keeps the AG Grid theme on its box and is as tall as its rows, so the whole book shows and the page scrolls', async () => {
    expect(CASHBOOK_UMAR_REAL.rows).toHaveLength(23);
    const list = await open();
    // Without the theme class AG Grid has no layout rules: the grid is 2 px tall and the table is blank.
    expect(list).toHaveClass(GRID_THEME_CLASS);
    // The cards above leave little room, so this book never takes the 'fill' height of a long list.
    expect(list).not.toHaveClass('list-grid-fill');
  });

  it("draws the lines of the answer, under the heading of each month, with the backend's totals", async () => {
    const list = await open();
    await waitFor(() => {
      expect(visible(list.textContent)).toContain('500,000');
    });
    const text = visible(list.textContent);
    // The two openings of the partners' capital, named for what they are with the partner (PRD FR-16.14).
    expect(text).toContain(
      [
        message(language, 'cashbook-row-kind-opening-capital'),
        language === 'ur' ? 'عدنان' : 'Adnan',
      ].join(message(language, 'cashbook-row-name-gap')),
    );
    // The month rows hold the backend's counts, not a count of the rows on screen.
    const headings = document.querySelectorAll('.month-heading');
    expect(headings).toHaveLength(2);
    expect(visible(headings[0]?.textContent)).toContain(
      message(language, 'common-month-count', { count: 2 }),
    );
    expect(visible(headings[1]?.textContent)).toContain(
      message(language, 'common-month-count', { count: 21 }),
    );
    // The totals row: in 1,758,340 and out 700,500, over the whole range.
    expect(text).toContain('1,758,340');
    expect(text).toContain('700,500');
    expect(visible(pageText())).toContain(rangeText(language, 1, 23, 23));
  });

  it('shows the capital and the cash at the start and at the end of the window as the backend sent them', async () => {
    await open();
    // The window starts before the first line (29-10-2026), so nothing was in hand: 0. It ends at 1,057,840, and
    // the partners put in 1,400,000.
    const capital = await screen.findByText(named(message(language, 'cashbook-card-capital')));
    const opening = await screen.findByText(named(message(language, 'cashbook-opening')));
    const closing = await screen.findByText(named(message(language, 'cashbook-closing')));
    // One card holds its label and its own figure.
    const cardOf = (label: HTMLElement) => visible(label.closest('dt')?.parentElement?.textContent);
    await waitFor(() => {
      expect(cardOf(capital)).toContain('1,400,000');
      expect(cardOf(opening)).toContain('0');
      expect(cardOf(closing)).toContain('1,057,840');
    });
    expect(cardOf(opening)).not.toContain('1,057,840');
  });
});
