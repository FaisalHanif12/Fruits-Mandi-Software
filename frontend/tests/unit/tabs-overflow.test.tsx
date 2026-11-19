// A row of tabs that is always ONE row (plan 38): the tabs that do not fit the free width are behind
// a "More" button with a menu, the open tab is always in the row, and before the row has a width
// every tab shows. The arithmetic of what fits is a plain function; the row is checked with a
// measured width. Urdu and English.

import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Localized } from '@fluent/react';
import { useState } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { AppProviders, createQueryClient } from '../../src/app/providers';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { Tabs, tabsThatFit } from '../../src/shared/ui/Tabs';
import { message } from '../helpers/render';

const IDS = ['one', 'two', 'three', 'four', 'five'] as const;

// The free width of the row and the width of each tab are measured by the browser; a test has no
// layout, so a test that needs them gives them (see below).
function Row({ label }: { label: string }) {
  const [value, setValue] = useState<string>('one');
  return (
    <Tabs
      overflow
      label={label}
      value={value}
      onValueChange={setValue}
      items={IDS.map((id) => ({
        id,
        label: <Localized id={`common-${id === 'one' ? 'back' : 'retry'}`} />,
        content: <p data-panel={id} />,
        menuLabel: id,
      }))}
    />
  );
}

describe('how many tabs fit', () => {
  it('shows all of them when the whole row fits, or when the row has no width yet', () => {
    expect(tabsThatFit(1000, [100, 100, 100], 80)).toBe(3);
    expect(tabsThatFit(0, [100, 100, 100], 80)).toBe(3);
  });

  it('keeps room for the More button and always shows one tab', () => {
    // 100 + 8 + 100 + 8 + 100 = 316 does not fit 250: the More button (80) takes room, 100 + 8 + 100
    // + 8 + 80 = 296 > 250, so one tab and More.
    expect(tabsThatFit(250, [100, 100, 100], 80)).toBe(1);
    expect(tabsThatFit(300, [100, 100, 100], 80)).toBe(2);
    expect(tabsThatFit(10, [100, 100, 100], 80)).toBe(1);
  });
});

describe.each(LANGUAGES)('the row of tabs (%s)', (language: Language) => {
  beforeEach(() => {
    act(() => {
      useLanguageStore.setState({ language });
    });
  });
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('shows every tab and no More button before the row has a width', () => {
    render(
      <AppProviders queryClient={createQueryClient()}>
        <Row label={message(language, 'settings-tabs')} />
      </AppProviders>,
    );
    expect(screen.getAllByRole('tab')).toHaveLength(5);
    expect(
      screen.queryByRole('button', { name: message(language, 'common-tabs-more') }),
    ).toBeNull();
  });

  it('puts the tabs that do not fit behind More, picks one from the menu, and keeps the open tab in the row', async () => {
    // A row 300 wide with tabs 100 wide and a More button 100 wide: one tab fits beside More.
    vi.spyOn(HTMLElement.prototype, 'clientWidth', 'get').mockReturnValue(300);
    vi.spyOn(HTMLElement.prototype, 'offsetWidth', 'get').mockReturnValue(100);
    const user = userEvent.setup();
    render(
      <AppProviders queryClient={createQueryClient()}>
        <Row label={message(language, 'settings-tabs')} />
      </AppProviders>,
    );
    await waitFor(() => {
      expect(screen.getAllByRole('tab').length).toBeLessThan(5);
    });
    const more = await screen.findByRole('button', { name: message(language, 'common-tabs-more') });
    // The first tab is open and in the row.
    expect(screen.getAllByRole('tab')[0]).toHaveAttribute('aria-selected', 'true');
    await user.click(more);
    const items = await screen.findAllByRole('menuitem');
    expect(items.length).toBeGreaterThan(0);
    const last = items.at(-1);
    if (last === undefined) throw new Error('no item');
    await user.click(last);
    // The tab that was picked is now open, and it is in the row (it took the place of another).
    await waitFor(() => {
      const open = screen
        .getAllByRole('tab')
        .filter((tab) => tab.getAttribute('aria-selected') === 'true');
      expect(open).toHaveLength(1);
    });
    expect(screen.getAllByRole('tab').length).toBeLessThan(5);
  });
});
