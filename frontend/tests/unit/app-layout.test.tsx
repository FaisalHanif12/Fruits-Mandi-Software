// The shell pieces (plan 04, PRD FR-5.2, FR-5.3, FR-5.5, FR-5.6, FR-7.9, FR-7.11, FR-7.17), in
// Urdu and English: main menu, side menu, top bar, status bar, stepper, the save prompt, text
// size and the frame that joins them.

import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { AppShell } from '../../src/app/layout/AppShell';
import { BIKRI_MENU, ModuleMenu } from '../../src/app/layout/ModuleMenu';
import { MAIN_NAV, MainNav } from '../../src/app/layout/MainNav';
import { StatusBar } from '../../src/app/layout/StatusBar';
import {
  DEFAULT_TEXT_SIZE,
  effectiveTextSize,
  maxTextSizeFor,
  stepTextSize,
  useTextSize,
  useTextSizeStore,
} from '../../src/app/layout/text-size';
import { TopBar } from '../../src/app/layout/TopBar';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { formNavigationProps } from '../../src/shared/keyboard';
import { SaveChangesPrompt, Stepper } from '../../src/shared/ui';
import { message, renderIn, visible } from '../helpers/render';
import { USER_NAME, at } from '../helpers/sample';

beforeEach(() => {
  useLanguageStore.setState({ language: 'ur' });
  useTextSizeStore.setState({ chosen: DEFAULT_TEXT_SIZE });
});

afterEach(() => {
  document.documentElement.style.zoom = '';
});

/** A name matcher that ignores the isolation marks Fluent puts around terms. */
const named = (language: Language, id: string) => (name: string) =>
  visible(name) === message(language, id);

function setLanguage(language: Language) {
  act(() => {
    useLanguageStore.setState({ language });
  });
}

describe.each(LANGUAGES)('MainNav (%s)', (language) => {
  it('lists the nine items of PRD FR-5.2 in order, with the current one marked', () => {
    setLanguage(language);
    renderIn(language, <MainNav current="bikri" onSelect={vi.fn()} />);
    const nav = screen.getByRole('navigation', { name: message(language, 'layout-nav-label') });
    const items = within(nav).getAllByRole('button');
    expect(items.map((item) => item.getAttribute('aria-label'))).toEqual(
      MAIN_NAV.map((entry) => message(language, entry.labelId)),
    );
    expect(MAIN_NAV.map((entry) => entry.id)).toEqual([
      'dashboard',
      'index',
      'bikri',
      'rokar',
      'partners',
      'analytics',
      'reports',
      'backup',
      'settings',
    ]);
    expect(items.filter((item) => item.getAttribute('aria-current') === 'page')).toHaveLength(1);
    expect(items[2]).toHaveAttribute('aria-current', 'page');
  });

  it('is a 64 px rail below 1366 px and 224 px from there, with labels only in the wide form', () => {
    renderIn(language, <MainNav current="dashboard" onSelect={vi.fn()} />);
    expect(screen.getByRole('navigation')).toHaveClass('w-rail', 'normal:w-nav');
    const label = within(at(screen.getAllByRole('button'), 0)).getByText(
      message(language, 'layout-nav-dashboard'),
    );
    expect(label).toHaveClass('hidden', 'normal:inline');
  });

  it('opens an item with Enter or a click, and ↑ ↓ move between items', async () => {
    const onSelect = vi.fn();
    renderIn(language, <MainNav current="dashboard" onSelect={onSelect} />);
    const user = userEvent.setup();
    const items = screen.getAllByRole('button');
    items[0]?.focus();
    await user.keyboard('{ArrowDown}{ArrowDown}');
    expect(items[2]).toHaveFocus();
    await user.keyboard('{Enter}');
    expect(onSelect).toHaveBeenCalledWith('bikri');
    await user.click(at(items, 4));
    expect(onSelect).toHaveBeenLastCalledWith('partners');
    await user.keyboard('{ArrowUp}');
    expect(items[3]).toHaveFocus();
  });
});

describe.each(LANGUAGES)('ModuleMenu (%s)', (language) => {
  it('lists the seven items of PRD FR-5.3 in order and marks the current one', () => {
    renderIn(language, <ModuleMenu current="receipt" onSelect={vi.fn()} />);
    const nav = screen.getByRole('navigation', { name: named(language, 'layout-module-label') });
    const items = within(nav).getAllByRole('button');
    expect(items.map((item) => visible(item.textContent))).toEqual(
      BIKRI_MENU.map((entry) => message(language, entry.labelId)),
    );
    expect(items[1]).toHaveAttribute('aria-current', 'page');
  });
});

describe.each(LANGUAGES)('TopBar (%s)', (language) => {
  it('Ctrl+K focuses the search box; the bell says "coming soon"', async () => {
    setLanguage(language);
    renderIn(
      language,
      <QueryClientProvider client={new QueryClient()}>
        <TopBar
          userName={USER_NAME}
          onLock={vi.fn()}
          onSignOut={vi.fn()}
          onOpenAccount={vi.fn()}
          onOpenBikri={vi.fn()}
          onOpenVoucher={vi.fn()}
        />
      </QueryClientProvider>,
    );
    const search = screen.getByRole('combobox', {
      name: message(language, 'layout-search-label'),
    });
    expect(search).not.toHaveFocus();
    await userEvent.setup().keyboard('{Control>}k{/Control}');
    expect(search).toHaveFocus();
    expect(
      screen.getByRole('button', { name: message(language, 'layout-attention-label') }),
    ).toHaveAttribute('title', message(language, 'layout-coming-soon'));
  });

  it('has the language switch and the user, shown as typed', () => {
    setLanguage(language);
    renderIn(
      language,
      <QueryClientProvider client={new QueryClient()}>
        <TopBar
          userName={USER_NAME}
          onLock={vi.fn()}
          onSignOut={vi.fn()}
          onOpenAccount={vi.fn()}
          onOpenBikri={vi.fn()}
          onOpenVoucher={vi.fn()}
        />
      </QueryClientProvider>,
    );
    expect(
      screen.getByRole('group', { name: message(language, 'common-language-switch') }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: message(language, 'layout-user-menu') }),
    ).toHaveTextContent(USER_NAME);
  });
});

describe.each(LANGUAGES)('StatusBar (%s)', (language) => {
  it('shows the user, the date left to right, and the day with a word as well as an icon', () => {
    renderIn(language, <StatusBar userName={USER_NAME} date="10-10-2026" day="open" />);
    const bar = screen.getByRole('contentinfo');
    expect(visible(bar.textContent)).toContain(message(language, 'layout-status-day-open'));
    expect(visible(bar.textContent)).toContain(USER_NAME);
    const date = bar.querySelector('bdi[dir="ltr"]');
    expect(date).toHaveTextContent('10-10-2026');
  });

  it('says when the day is closed', () => {
    renderIn(language, <StatusBar userName={USER_NAME} date="10-10-2026" day="closed" />);
    expect(visible(screen.getByRole('contentinfo').textContent)).toContain(
      message(language, 'layout-status-day-closed'),
    );
  });
});

describe.each(LANGUAGES)('Stepper (%s)', (language) => {
  it('shows done steps with a tick, the current step with its number, and the rest plain', () => {
    renderIn(
      language,
      <Stepper
        label={message(language, 'layout-stepper-label')}
        current={1}
        steps={[
          { id: 'a', label: message(language, 'layout-nav-reports') },
          { id: 'b', label: message(language, 'layout-nav-backup') },
          { id: 'c', label: message(language, 'layout-nav-settings') },
        ]}
      />,
    );
    const items = within(
      screen.getByRole('list', { name: message(language, 'layout-stepper-label') }),
    ).getAllByRole('listitem');
    expect(items).toHaveLength(3);
    expect(items[0]?.querySelector('svg')).not.toBeNull();
    expect(items[0]).not.toHaveAttribute('aria-current');
    expect(items[1]).toHaveAttribute('aria-current', 'step');
    expect(items[1]).toHaveTextContent('2');
    expect(items[2]).not.toHaveAttribute('aria-current');
    expect(items[2]).toHaveTextContent('3');
  });
});

describe.each(LANGUAGES)('SaveChangesPrompt (%s)', (language) => {
  it('has three buttons, save is focused, and each does what it says', async () => {
    const onSave = vi.fn();
    const onLeave = vi.fn();
    const onOpenChange = vi.fn();
    renderIn(
      language,
      <SaveChangesPrompt open onOpenChange={onOpenChange} onSave={onSave} onLeave={onLeave} />,
    );
    const dialog = await screen.findByRole('alertdialog');
    expect(within(dialog).getAllByRole('button')).toHaveLength(3);
    const save = within(dialog).getByRole('button', {
      name: new RegExp(message(language, 'layout-save-save')),
    });
    expect(save).toHaveFocus();
    expect(dialog).toHaveTextContent(message(language, 'layout-save-title'));
    await userEvent
      .setup()
      .click(within(dialog).getByRole('button', { name: message(language, 'layout-save-leave') }));
    expect(onLeave).toHaveBeenCalledTimes(1);
    expect(onSave).not.toHaveBeenCalled();
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });

  it('Enter saves, because save has focus', async () => {
    const onSave = vi.fn();
    renderIn(
      language,
      <SaveChangesPrompt open onOpenChange={vi.fn()} onSave={onSave} onLeave={vi.fn()} />,
    );
    await screen.findByRole('alertdialog');
    await userEvent.setup().keyboard('{Enter}');
    expect(onSave).toHaveBeenCalledTimes(1);
  });

  it('Esc and the back button close it without doing anything else', async () => {
    const onOpenChange = vi.fn();
    const onSave = vi.fn();
    const onLeave = vi.fn();
    renderIn(
      language,
      <SaveChangesPrompt open onOpenChange={onOpenChange} onSave={onSave} onLeave={onLeave} />,
    );
    await screen.findByRole('alertdialog');
    await userEvent.setup().keyboard('{Escape}');
    expect(onOpenChange).toHaveBeenCalledWith(false);
    expect(onSave).not.toHaveBeenCalled();
    expect(onLeave).not.toHaveBeenCalled();
  });
});

describe('text size (PRD FR-7.11, NFR-USE-18)', () => {
  it.each([
    [1024, 100],
    [1100, 100],
    [1366, 130],
    [1536, 150],
    [1920, 150],
    [800, 100],
  ])('at %i points wide the largest step is %i%%', (width, expected) => {
    expect(maxTextSizeFor(width)).toBe(expected);
  });

  it('goes up and down by 10, between 80 and 150, and never past what the window allows', () => {
    expect(stepTextSize(100, 1, 1920)).toBe(110);
    expect(stepTextSize(100, -1, 1920)).toBe(90);
    expect(stepTextSize(80, -1, 1920)).toBe(80);
    expect(stepTextSize(150, 1, 1920)).toBe(150);
    expect(stepTextSize(130, 1, 1366)).toBe(130);
    expect(stepTextSize(100, 1, 1024)).toBe(100);
  });

  it('holds a big choice back in a narrow window, without forgetting it', () => {
    expect(effectiveTextSize(150, 1366)).toBe(130);
    expect(effectiveTextSize(150, 1920)).toBe(150);
    expect(effectiveTextSize(80, 1024)).toBe(80);
  });

  function Mount() {
    useTextSize();
    return null;
  }

  const press = (code: string, init: KeyboardEventInit = {}) => {
    act(() => {
      fireEvent.keyDown(window, { code, ctrlKey: true, ...init });
    });
  };

  it('Ctrl +, Ctrl − and Ctrl 0 change the zoom of the page', () => {
    Object.defineProperty(window, 'innerWidth', { value: 1920, configurable: true });
    render(<Mount />);
    expect(document.documentElement.style.zoom).toBe('1');
    press('Equal', { shiftKey: true });
    expect(document.documentElement.style.zoom).toBe('1.1');
    press('NumpadAdd');
    press('Equal');
    expect(document.documentElement.style.zoom).toBe('1.3');
    press('Minus');
    expect(document.documentElement.style.zoom).toBe('1.2');
    press('Digit0');
    expect(document.documentElement.style.zoom).toBe('1');
    for (let step = 0; step < 6; step += 1) press('NumpadSubtract');
    expect(document.documentElement.style.zoom).toBe('0.8');
  });

  it('stops where the content would be narrower than 1024 points, and follows a resize', () => {
    Object.defineProperty(window, 'innerWidth', { value: 1366, configurable: true });
    useTextSizeStore.setState({ chosen: 150 });
    render(<Mount />);
    expect(document.documentElement.style.zoom).toBe('1.3');
    press('Equal');
    expect(document.documentElement.style.zoom).toBe('1.3');
    Object.defineProperty(window, 'innerWidth', { value: 1024, configurable: true });
    act(() => {
      fireEvent(window, new Event('resize'));
    });
    expect(document.documentElement.style.zoom).toBe('1');
  });
});

function SampleScreen() {
  return (
    <form {...formNavigationProps}>
      <input data-testid="first" />
      <input data-testid="second" />
    </form>
  );
}

describe.each(LANGUAGES)('AppShell (%s)', (language) => {
  function renderShell() {
    setLanguage(language);
    return renderIn(
      language,
      <QueryClientProvider client={new QueryClient()}>
        <AppShell
          nav={<MainNav current="bikri" onSelect={vi.fn()} />}
          moduleMenu={<ModuleMenu current="bikri-book" onSelect={vi.fn()} />}
          userName={USER_NAME}
          onLock={vi.fn()}
          onSignOut={vi.fn()}
          onOpenAccount={vi.fn()}
          onOpenBikri={vi.fn()}
          onOpenVoucher={vi.fn()}
          date="10-10-2026"
          day="open"
        >
          <SampleScreen />
        </AppShell>
      </QueryClientProvider>,
    );
  }

  it('is as tall as the window and clipped, and the screen is the containing block, so the page never scrolls itself', () => {
    renderShell();
    // Layout cannot be measured here; the classes that keep the page still are checked
    // (D1+D2 review, finding 29, measured in a browser): the shell clips, the screen scrolls, and
    // the screen holds the hidden native <select> Radix puts beside each drop-down.
    const main = screen.getByRole('main');
    expect(main).toHaveClass('relative', 'overflow-auto');
    expect(main.closest('.h-screen')).toHaveClass('overflow-hidden');
  });

  // The menu is on the start side: the right in Urdu, the left in English.
  const intoScreen = language === 'ur' ? '{ArrowLeft}' : '{ArrowRight}';
  const toMenu = language === 'ur' ? '{ArrowRight}' : '{ArrowLeft}';

  it('puts the menus, the top bar, the screen and the status bar in one frame', () => {
    renderShell();
    expect(screen.getAllByRole('navigation')).toHaveLength(2);
    expect(screen.getByRole('banner')).toBeInTheDocument();
    expect(screen.getByRole('main')).toContainElement(screen.getByTestId('first'));
    expect(screen.getByRole('contentinfo')).toBeInTheDocument();
  });

  it('the arrow into the screen leaves the menu for the screen, and the other goes back', async () => {
    renderShell();
    const user = userEvent.setup();
    const sideMenuItems = within(
      screen.getByRole('navigation', { name: named(language, 'layout-module-label') }),
    ).getAllByRole('button');
    const sideMenuItem = at(sideMenuItems, 0);
    sideMenuItem.focus();
    await user.keyboard(intoScreen);
    expect(screen.getByTestId('first')).toHaveFocus();
    // From the first field, the way back is the menu side. The nearest menu answers.
    await user.keyboard(toMenu);
    expect(sideMenuItem).toHaveFocus();
  });
});
