// Plan 93 (PRD FR-5.3 and FR-7.17, v1.56): the second menu of an area is a row of tabs in the title row of the
// screen. One shared component for every section: icon and short label, the open tab marked and the page title,
// always ONE row (the tabs that do not fit go behind "More", the open one stays in the row), roving focus with ←
// and → in the reading direction, and a screen outside a section keeps its plain title. In the Urdu and in the
// English app. The arithmetic of what fits is a plain function; the row is checked with a measured width.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Receipt, Wallet, BookText } from 'lucide-react';
import { useState } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ModuleTabs, RokarTabs } from '../../src/app/layout/ModuleMenu';
import { LANGUAGES, type Language } from '../../src/i18n';
import {
  ListPage,
  SectionTabs,
  SectionTabsProvider,
  TitleOrTabs,
  sectionTabsThatFit,
} from '../../src/shared/ui';
import { message, renderIn, visible } from '../helpers/render';
import { at } from '../helpers/sample';

afterEach(() => {
  vi.restoreAllMocks();
});

describe('how many tabs fit', () => {
  it('shows all of them when the whole row fits, or when the row has no width yet', () => {
    expect(sectionTabsThatFit(1000, [100, 100, 100], 80)).toBe(3);
    expect(sectionTabsThatFit(0, [100, 100, 100], 80)).toBe(3);
  });

  it('keeps room for the More button and always shows one tab', () => {
    // The gap between two tabs is 4: 100 + 4 + 100 + 4 + 100 = 308 does not fit 250.
    expect(sectionTabsThatFit(250, [100, 100, 100], 80)).toBe(1);
    expect(sectionTabsThatFit(300, [100, 100, 100], 80)).toBe(2);
    expect(sectionTabsThatFit(10, [100, 100, 100], 80)).toBe(1);
  });
});

const named = (text: string) => (name: string) => visible(name) === text;

describe.each(LANGUAGES)('the tabs of a section (%s)', (language: Language) => {
  const m = (id: string) => message(language, id);

  /** A row with three tabs and the open one chosen by the test. */
  function Row({ onSelect = vi.fn() }: { onSelect?: (id: 'a' | 'b' | 'c') => void }) {
    const [current, setCurrent] = useState<'a' | 'b' | 'c'>('a');
    return (
      <SectionTabs
        label={m('layout-module-label')}
        current={current}
        onSelect={(id) => {
          setCurrent(id);
          onSelect(id);
        }}
        tabs={[
          { id: 'a', label: m('layout-module-bikri-book'), icon: Receipt },
          { id: 'b', label: m('layout-module-receipt'), icon: Wallet },
          { id: 'c', label: m('layout-module-payment'), icon: BookText },
        ]}
      />
    );
  }

  const tabsOf = () =>
    within(screen.getByRole('navigation', { name: named(m('layout-module-label')) })).getAllByRole(
      'button',
    );

  it('shows every tab with its words, marks the open one, and is one row with no More button before it has a width', () => {
    renderIn(language, <Row />);
    const tabs = tabsOf();
    expect(tabs.map((tab) => visible(tab.textContent))).toEqual([
      m('layout-module-bikri-book'),
      m('layout-module-receipt'),
      m('layout-module-payment'),
    ]);
    expect(at(tabs, 0)).toHaveAttribute('aria-current', 'page');
    expect(at(tabs, 1)).not.toHaveAttribute('aria-current');
    // The open tab is a filled pill in primary; the others are quiet.
    expect(at(tabs, 0)).toHaveClass('bg-primary', 'text-on-primary');
    expect(at(tabs, 1)).not.toHaveClass('bg-primary');
    // Never two lines: no tab wraps its words.
    for (const tab of tabs) expect(tab).toHaveClass('whitespace-nowrap');
    expect(screen.queryByRole('button', { name: m('common-tabs-more') })).toBeNull();
  });

  it('the tabs that do not fit go behind More, the open tab stays in the row, and a pick from the menu opens it', async () => {
    // A row 250 wide with tabs 100 wide and a More button 100 wide: one tab fits beside More.
    vi.spyOn(HTMLElement.prototype, 'clientWidth', 'get').mockReturnValue(250);
    vi.spyOn(HTMLElement.prototype, 'offsetWidth', 'get').mockReturnValue(100);
    const onSelect = vi.fn();
    renderIn(language, <Row onSelect={onSelect} />);
    const user = userEvent.setup();
    // Only the open tab and More are in the row.
    await waitFor(() => {
      expect(tabsOf()).toHaveLength(2);
    });
    expect(visible(at(tabsOf(), 0).textContent)).toBe(m('layout-module-bikri-book'));
    const more = await screen.findByRole('button', { name: named(m('common-tabs-more')) });
    await user.click(more);
    const items = await screen.findAllByRole('menuitem');
    expect(items.map((item) => visible(item.textContent))).toEqual([
      m('layout-module-receipt'),
      m('layout-module-payment'),
    ]);
    await user.click(at(items, 1));
    expect(onSelect).toHaveBeenCalledWith('c');
    // The picked tab is now the open one and is in the row: it took the place of the last tab that fits.
    await waitFor(() => {
      const open = tabsOf().find((tab) => tab.getAttribute('aria-current') === 'page');
      expect(visible(open?.textContent)).toBe(m('layout-module-payment'));
    });
  });

  it('the bikri and the rokar tabs are the same component with their own items (PRD FR-5.3, 5)', () => {
    const first = renderIn(language, <ModuleTabs current="today" onSelect={vi.fn()} />);
    expect(tabsOf()).toHaveLength(7);
    first.unmount();
    renderIn(language, <RokarTabs current="expense" onSelect={vi.fn()} />);
    const rokar = within(
      screen.getByRole('navigation', { name: named(m('layout-rokar-label')) }),
    ).getAllByRole('button');
    expect(rokar.map((tab) => visible(tab.textContent))).toEqual([
      m('layout-rokar-cashbook'),
      m('layout-rokar-expense'),
      m('layout-rokar-lent'),
      m('layout-rokar-khata'),
      m('layout-rokar-general'),
      m('layout-rokar-voucher'),
    ]);
    expect(at(rokar, 1)).toHaveAttribute('aria-current', 'page');
  });

  it('a screen in a section draws the tabs in its title row with its buttons at the end, and keeps its heading for readers', () => {
    renderIn(
      language,
      <SectionTabsProvider value={<ModuleTabs current="today" onSelect={vi.fn()} />}>
        <ListPage
          titleId="layout-module-today"
          actions={<button type="button">{m('common-back')}</button>}
        >
          <p>{m('common-retry')}</p>
        </ListPage>
      </SectionTabsProvider>,
    );
    expect(tabsOf()).toHaveLength(7);
    const heading = screen.getByRole('heading', { level: 1 });
    expect(visible(heading.textContent)).toBe(m('layout-module-today'));
    expect(heading).toHaveClass('sr-only');
    // The tabs and the screen's button are in one row.
    const row = tabsOf()[0]?.closest('nav')?.parentElement?.parentElement;
    expect(row).toContainElement(screen.getByRole('button', { name: m('common-back') }));
  });

  it('the row wants the width of ALL its tabs, so a title row wraps its buttons under the tabs instead of hiding tabs', () => {
    renderIn(
      language,
      <SectionTabsProvider value={<ModuleTabs current="bikri-book" onSelect={vi.fn()} />}>
        <ListPage
          titleId="layout-module-today"
          actions={<button type="button">{m('common-back')}</button>}
        >
          <p>{m('common-retry')}</p>
        </ListPage>
      </SectionTabsProvider>,
    );
    const nav = screen.getByRole('navigation', { name: named(m('layout-module-label')) });
    // An in-flow ruler with no height holds every tab: the row's natural width is the width of all of them, whatever is
    // shown now (so the title row can decide, by wrapping, whether the buttons fit beside it).
    const ruler = nav.querySelector('[aria-hidden="true"]');
    expect(ruler).toHaveClass('h-0', 'invisible', 'overflow-hidden');
    expect(ruler?.firstElementChild?.children).toHaveLength(7);
    // The name's block grows from its natural width (all tabs) and gives way only when the tabs alone do not fit; the
    // screen's buttons keep their size and sit at the end of whichever row they land on.
    const block = nav.parentElement;
    expect(block).toHaveClass('flex-auto', 'min-w-0');
    expect(nav).toHaveClass('shrink', 'min-w-0');
    const buttons = screen.getByRole('button', { name: m('common-back') }).parentElement;
    expect(buttons).toHaveClass('ms-auto', 'shrink-0');
    expect(block?.parentElement).toHaveClass('flex-wrap');
  });

  it('a screen that belongs to no section keeps its own visible title', () => {
    renderIn(
      language,
      <>
        <ListPage titleId="layout-module-today">
          <p>{m('common-retry')}</p>
        </ListPage>
        <TitleOrTabs title={m('common-back')}>
          <p>{m('common-loading')}</p>
        </TitleOrTabs>
      </>,
    );
    const heading = screen.getByRole('heading', { level: 1 });
    expect(heading).not.toHaveClass('sr-only');
    expect(screen.getByText(m('common-loading'))).toBeInTheDocument();
    expect(screen.queryByRole('navigation')).toBeNull();
  });
});
