// The one window every list shares (plan 76 point 8, PRD FR-19.11): `useListWindow` and `ListWindowBar`, in the
// Urdu and the English app. A list with many rows a day shows 7 days, a list with few rows a day one month;
// it opens on the latest window, moves by the buttons and the typed day or month, and never passes today nor
// goes before the first day the backend says has data (`windows_bounds`). The date work is in
// days-window.test.ts and format-months.test.ts; here are the range the list is asked for, the buttons that
// go off at the two ends, the box, and the Page Up and Page Down hand-over (`onPageKey`).

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { WindowSource } from '../../src/bindings/WindowSource';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useListWindow, type ListWindowKind } from '../../src/shared/hooks';
import { ListWindowBar } from '../../src/shared/ui';
import * as windowsApi from '../../src/shared/windows/api';
import { message, renderIn, visible } from '../helpers/render';

vi.mock('../../src/shared/windows/api');

const TODAY = '2026-10-10';
const SOURCE: WindowSource = { kind: 'cashbook' };

function Harness({ kind }: { kind: ListWindowKind }) {
  const listWindow = useListWindow({ kind, source: SOURCE, today: TODAY });
  return (
    <div>
      <p data-testid="range">{`${listWindow.range.from}|${listWindow.range.to}`}</p>
      <button
        type="button"
        data-testid="page-up"
        onClick={() => {
          listWindow.onPageKey('previous');
        }}
      />
      <ListWindowBar window={listWindow} />
    </div>
  );
}

function draw(language: Language, kind: ListWindowKind) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return renderIn(
    language,
    <QueryClientProvider client={client}>
      <Harness kind={kind} />
    </QueryClientProvider>,
  );
}

const range = () => screen.getByTestId('range').textContent;

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(windowsApi.windowsBounds).mockResolvedValue({
    first_date: '2026-09-20',
    last_date: TODAY,
  });
});

describe.each(LANGUAGES)('a window of 7 days (%s)', (language) => {
  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const button = (id: string) =>
    screen.getByRole('button', { name: (name) => visible(name).includes(m(id, { days: '7' })) });

  it('opens on the 7 days that end today, and asks the backend for the first day of the list', async () => {
    draw(language, 'days');
    await waitFor(() => {
      expect(range()).toBe('04-10-2026|10-10-2026');
    });
    expect(windowsApi.windowsBounds).toHaveBeenCalledWith({ source: SOURCE });
    // The later button is off and says why; the earlier one shows the days it goes to.
    expect((button('common-days-later') as HTMLButtonElement).disabled).toBe(true);
    expect(screen.getByText(m('common-days-latest-note'))).toBeInTheDocument();
  });

  it('the earlier button moves by 7 days, the later one comes back, and Page Up does the same as the earlier one', async () => {
    const user = userEvent.setup();
    draw(language, 'days');
    await waitFor(() => {
      expect(range()).toBe('04-10-2026|10-10-2026');
    });
    await user.click(button('common-days-earlier'));
    expect(range()).toBe('27-09-2026|03-10-2026');
    await user.click(button('common-days-later'));
    expect(range()).toBe('04-10-2026|10-10-2026');
    await user.click(screen.getByTestId('page-up'));
    expect(range()).toBe('27-09-2026|03-10-2026');
  });

  it('stops at the first window that holds the first day of data, and says so', async () => {
    const user = userEvent.setup();
    draw(language, 'days');
    await waitFor(() => {
      expect(range()).toBe('04-10-2026|10-10-2026');
    });
    await user.click(button('common-days-earlier'));
    await user.click(button('common-days-earlier'));
    // The first day is 20-09-2026: the window that holds it ends on 26-09-2026.
    expect(range()).toBe('20-09-2026|26-09-2026');
    expect((button('common-days-earlier') as HTMLButtonElement).disabled).toBe(true);
    expect(screen.getByText(m('common-days-first-note'))).toBeInTheDocument();
  });

  it('a typed day goes to the window that ends on it, and a day after today is refused in words', async () => {
    const user = userEvent.setup();
    draw(language, 'days');
    await waitFor(() => {
      expect(range()).toBe('04-10-2026|10-10-2026');
    });
    const box = screen.getByLabelText<HTMLInputElement>((name) =>
      visible(name).includes(m('common-goto-day')),
    );
    await user.type(box, '5{Enter}');
    await waitFor(() => {
      expect(range()).toBe('29-09-2026|05-10-2026');
    });
    // A button leads back to the latest window.
    await user.click(screen.getByRole('button', { name: m('common-days-latest-button') }));
    expect(range()).toBe('04-10-2026|10-10-2026');
  });

  it('waits for the day: with no day there is no range and no bar', () => {
    const client = new QueryClient();
    function Waiting() {
      const listWindow = useListWindow({ kind: 'days', source: null, today: undefined });
      return (
        <div>
          <p data-testid="range">{`${listWindow.range.from}|${listWindow.range.to}`}</p>
          <ListWindowBar window={listWindow} />
        </div>
      );
    }
    renderIn(
      language,
      <QueryClientProvider client={client}>
        <Waiting />
      </QueryClientProvider>,
    );
    expect(range()).toBe('|');
    expect(screen.queryByRole('group')).toBeNull();
    expect(windowsApi.windowsBounds).not.toHaveBeenCalled();
  });
});

describe.each(LANGUAGES)('a window of one month (%s)', (language) => {
  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);

  it('opens on this month, goes back one month, and stops at the first month with data', async () => {
    vi.mocked(windowsApi.windowsBounds).mockResolvedValue({
      first_date: '2026-09-20',
      last_date: TODAY,
    });
    const user = userEvent.setup();
    draw(language, 'month');
    await waitFor(() => {
      expect(range()).toBe('01-10-2026|31-10-2026');
    });
    await user.click(
      screen.getByRole('button', {
        name: (name) => visible(name).includes(m('common-month-earlier')),
      }),
    );
    expect(range()).toBe('01-09-2026|30-09-2026');
    expect(screen.getByText(m('common-month-first-note'))).toBeInTheDocument();
    await user.click(screen.getByTestId('page-up'));
    // September is the first month: Page Up does not go before it.
    expect(range()).toBe('01-09-2026|30-09-2026');
  });

  it('a typed month goes to it', async () => {
    const user = userEvent.setup();
    draw(language, 'month');
    await waitFor(() => {
      expect(range()).toBe('01-10-2026|31-10-2026');
    });
    const box = screen.getByLabelText<HTMLInputElement>((name) =>
      visible(name).includes(m('common-goto-month')),
    );
    await user.type(box, '9{Enter}');
    await waitFor(() => {
      expect(range()).toBe('01-09-2026|30-09-2026');
    });
  });
});
