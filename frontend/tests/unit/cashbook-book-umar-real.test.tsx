// The روکڑ بک with Umar's own books (plan 96 T2, PRD FR-16.14, FR-19.11). The answer is what the real backend
// sent for the window of 30 days that ends 24-11-2026: 11 lines, so the list is a long one. A list of more
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
    prepareCashbook(language, [], { cash: 924260 });
    vi.mocked(cashbookApi.cashbookGet).mockResolvedValue(CASHBOOK_UMAR_REAL);
    await renderApp('/rokar');
    await screen.findByRole('heading', { level: 1, name: message(language, 'cashbook-title') });
    return screen.findByRole('group', { name: named(message(language, 'cashbook-list-label')) });
  }

  it('keeps the AG Grid theme on its box and is as tall as its rows, so the whole book shows and the page scrolls', async () => {
    expect(CASHBOOK_UMAR_REAL.rows).toHaveLength(11);
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
    // The two openings of the partners' capital, with the partner's name (plan 78 E).
    expect(text).toContain(
      message(language, 'cashbook-row-opening-capital', {
        partner: language === 'ur' ? 'عدنان' : 'Adnan',
      }),
    );
    // The month rows hold the backend's counts, not a count of the rows on screen.
    const headings = document.querySelectorAll('.month-heading');
    expect(headings).toHaveLength(2);
    expect(visible(headings[0]?.textContent)).toContain(
      message(language, 'common-month-count', { count: 2 }),
    );
    expect(visible(headings[1]?.textContent)).toContain(
      message(language, 'common-month-count', { count: 9 }),
    );
    // The totals row: in 1,335,620 and out 411,360, over the whole range.
    expect(text).toContain('1,335,620');
    expect(text).toContain('411,360');
    expect(visible(pageText())).toContain(rangeText(language, 1, 11, 11));
  });

  it('shows the cash at the start and at the end of the window as the backend sent them', async () => {
    await open();
    // The window starts before the first line (29-10-2026), so nothing was in hand: 0. It ends at 924,260.
    const opening = await screen.findByText(message(language, 'cashbook-opening'));
    const closing = await screen.findByText(message(language, 'cashbook-closing'));
    const factOf = (label: HTMLElement) => visible(label.closest('dl')?.textContent);
    await waitFor(() => {
      expect(factOf(opening)).toContain('0');
      expect(factOf(closing)).toContain('924,260');
    });
    expect(factOf(opening)).not.toContain('924,260');
  });
});
