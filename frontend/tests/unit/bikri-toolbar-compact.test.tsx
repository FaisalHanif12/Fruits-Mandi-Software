// Plan 95 (PRD FR-5.3 v1.57, Umar 23-11-2026): the top row of the sale book keeps every tab in the row. When the
// controls leave the tabs too little room, the print buttons show an icon and a short word with no key chip, and say
// their key in the tip and the name;
// only if that is still not enough do the last tabs go behind "More". The measuring is done in the browser (WebKit,
// checked by hand at 1024, 1366, 1480 and 1920); here are the rule, the words kept for readers, and the row's
// attribute. In the Urdu and in the English app.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { BikriToolbar, tabsNeedRoom } from '../../src/features/bikri/components/BikriToolbar';
import { CompactControlsProvider } from '../../src/features/bikri/components/compact-controls';
import { PrintAction } from '../../src/features/bikri/print/PrintAction';
import { LANGUAGES } from '../../src/i18n';
import { SectionTabs, SectionTabsProvider, TitleOrTabs } from '../../src/shared/ui';
import { BookText, Receipt, Wallet } from 'lucide-react';
import { message, renderIn, visible } from '../helpers/render';

afterEach(() => {
  vi.restoreAllMocks();
});

describe('when the tabs need room', () => {
  it('only when they have less than they want, by more than a pixel of rounding', () => {
    expect(tabsNeedRoom(430, 431)).toBe(false);
    expect(tabsNeedRoom(443, 430.4)).toBe(false);
    expect(tabsNeedRoom(411, 430.4)).toBe(true);
    // No tabs measured yet (a test, a window being built): nothing to make room for.
    expect(tabsNeedRoom(100, 0)).toBe(false);
  });
});

describe.each(LANGUAGES)('the print buttons of the sale book (%s)', (language) => {
  const m = (id: string) => message(language, id);

  it('show their words and their key as always, with no tip, while the row has room', () => {
    renderIn(language, <PrintAction target={null} />);
    const print = screen.getByRole('button', {
      name: (name) => visible(name).includes(m('print-box-print')),
    });
    expect(print).not.toHaveAttribute('title');
    // The long word is drawn; only the row's attribute swaps it for the short one (`group-data-[compact]/toolbar:`).
    const label = [...print.querySelectorAll('span')].find((span) =>
      visible(span.textContent).includes(m('print-box-print')),
    );
    expect(label?.className).toContain('group-data-[compact]/toolbar:hidden');
    const short = [...print.querySelectorAll('span')].find((span) =>
      span.className.includes('group-data-[compact]/toolbar:inline'),
    );
    expect(visible(short?.textContent)).toBe(m('print-box-print-short'));
    expect(short?.className).toContain('hidden');
    // The key chip is there, and is the one thing the short row takes away.
    const chip = print.querySelector('kbd');
    expect(chip?.textContent).toBe('F9');
    expect(chip?.className).toContain('group-data-[compact]/toolbar:hidden');
  });

  it('say their words and their key in the tip and the name when the row is short of room', () => {
    renderIn(
      language,
      <CompactControlsProvider value>
        <PrintAction target={null} />
      </CompactControlsProvider>,
    );
    for (const [words, key] of [
      [m('print-box-print'), 'F9'],
      [m('report-r5-title'), 'Shift+F9'],
    ] as const) {
      const button = screen.getByTitle((title) => visible(title) === `${words} (${key})`);
      expect(visible(button.getAttribute('aria-label'))).toBe(`${words} (${key})`);
      // The padding is half as wide in the short row, so an icon and a word take as little as they can.
      expect(button.className).toContain('group-data-[compact]/toolbar:px-2');
      expect(button.querySelector('kbd')?.className).toContain(
        'group-data-[compact]/toolbar:hidden',
      );
    }
  });

  it('the short word of print is shorter than its long word in Urdu, and the adhari button keeps its own words', () => {
    if (language === 'ur') {
      expect(m('print-box-print-short').length).toBeLessThan(m('print-box-print').length);
    }
    renderIn(language, <PrintAction target={null} />);
    const adhari = screen.getByRole('button', {
      name: (name) => visible(name).includes(m('report-r5-title')),
    });
    expect(adhari.querySelector('kbd')?.textContent).toBe('Shift+F9');
    expect(adhari.querySelector('kbd')?.className).toContain('group-data-[compact]/toolbar:hidden');
  });
});

describe.each(LANGUAGES)('the top row of the sale book (%s)', (language) => {
  it('is the group the print buttons follow, and no row is marked until the browser has measured the tabs', () => {
    renderIn(
      language,
      <QueryClientProvider client={new QueryClient()}>
        <SectionTabsProvider
          value={
            <SectionTabs
              label={message(language, 'layout-module-label')}
              current="a"
              onSelect={vi.fn()}
              tabs={[
                { id: 'a', label: message(language, 'layout-module-bikri-book'), icon: Receipt },
                { id: 'b', label: message(language, 'layout-module-party-ledgers'), icon: Wallet },
                { id: 'c', label: message(language, 'layout-module-today'), icon: BookText },
              ]}
            />
          }
        >
          <BikriToolbar
            onOpen={vi.fn()}
            findOpen={false}
            onFindOpenChange={vi.fn()}
            title={
              <TitleOrTabs title={message(language, 'layout-module-bikri-book')}>
                <h1>{message(language, 'layout-module-bikri-book')}</h1>
              </TitleOrTabs>
            }
          >
            <PrintAction target={null} />
          </BikriToolbar>
        </SectionTabsProvider>
      </QueryClientProvider>,
    );
    const nav = screen.getByRole('navigation', {
      name: (name) => visible(name) === message(language, 'layout-module-label'),
    });
    const row = nav.closest('.group\\/toolbar');
    expect(row).not.toBeNull();
    // With no width to measure (jsdom) every tab is shown and the controls keep their words.
    expect(row).not.toHaveAttribute('data-compact');
    // The row writes how wide all its tabs want to be, for the screen's own controls to read.
    expect(nav).toHaveAttribute('data-natural-width');
  });
});
