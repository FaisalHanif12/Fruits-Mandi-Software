// The shortcut registry (PRD 7.3, Code Standards F29) and the Esc order (PRD FR-7.18).

import { fireEvent, render } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  ALL_SHORTCUTS,
  SHORTCUTS,
  intoScreenSide,
  matchesShortcut,
  menuSide,
  registerEscape,
  runEscape,
  useEscapeDispatcher,
  type Shortcut,
} from '../../src/shared/keyboard';

function press(code: string, modifiers: Partial<KeyboardEventInit> = {}): KeyboardEvent {
  return new KeyboardEvent('keydown', { code, ...modifiers });
}

// PRD 7.3, row by row: the label the button shows.
const PRD_KEYS: readonly (readonly [keyof typeof SHORTCUTS, string])[] = [
  ['newEntry', 'F2'],
  ['find', 'F3'],
  ['addAccount', 'F4'],
  ['print', 'F9'],
  ['save', 'F10'],
  ['saveAndNext', 'Ctrl+Enter'],
  ['removeRow', 'Ctrl+Delete'],
  ['typingMode', 'Alt+U'],
  ['languageToggle', 'Alt+L'],
  ['globalSearch', 'Ctrl+K'],
  ['explain', 'Ctrl+I'],
  ['copyAbove', 'Ctrl+D'],
  ['undoSave', 'Alt+Z'],
  ['textBigger', 'Ctrl +'],
  ['textSmaller', 'Ctrl −'],
  ['textNormal', 'Ctrl 0'],
  ['newBikri', 'Alt+B'],
  ['newReceipt', 'Alt+V'],
  ['newPayment', 'Alt+A'],
  ['newAdvance', 'Alt+P'],
  ['newExpense', 'Alt+K'],
  ['back', 'Esc'],
  ['openList', 'Alt+↓'],
  ['dateNextDay', '+'],
  ['datePreviousDay', '−'],
];

describe('shortcut registry', () => {
  it.each(PRD_KEYS)('%s is registered with the label %s', (name, label) => {
    expect(SHORTCUTS[name].label).toBe(label);
  });

  it('has nothing besides the PRD keys', () => {
    expect(ALL_SHORTCUTS).toHaveLength(PRD_KEYS.length);
  });

  it('gives every key combination to one shortcut only', () => {
    const seen = new Map<string, string>();
    for (const shortcut of ALL_SHORTCUTS) {
      for (const key of [shortcut, ...(shortcut.also ?? [])]) {
        const shifts = key.shift === 'any' ? [false, true] : [key.shift ?? false];
        for (const shift of shifts) {
          const id = [key.code, key.alt ?? false, key.ctrl ?? false, shift].join('|');
          expect(seen.get(id), `${shortcut.label} clashes with ${seen.get(id) ?? ''}`).toBe(
            undefined,
          );
          seen.set(id, shortcut.label);
        }
      }
    }
  });

  it('matches by the physical key and the exact modifiers', () => {
    expect(matchesShortcut(press('KeyL', { altKey: true }), SHORTCUTS.languageToggle)).toBe(true);
    expect(matchesShortcut(press('KeyL'), SHORTCUTS.languageToggle)).toBe(false);
    expect(
      matchesShortcut(press('KeyL', { altKey: true, ctrlKey: true }), SHORTCUTS.languageToggle),
    ).toBe(false);
    expect(
      matchesShortcut(press('KeyL', { altKey: true, metaKey: true }), SHORTCUTS.languageToggle),
    ).toBe(false);
  });

  it('save answers F10 and Ctrl+S', () => {
    expect(matchesShortcut(press('F10'), SHORTCUTS.save)).toBe(true);
    expect(matchesShortcut(press('KeyS', { ctrlKey: true }), SHORTCUTS.save)).toBe(true);
    expect(matchesShortcut(press('KeyS'), SHORTCUTS.save)).toBe(false);
  });

  it('text size answers the main keys, the number pad, and the plus key with Shift', () => {
    const bigger: Shortcut = SHORTCUTS.textBigger;
    expect(matchesShortcut(press('Equal', { ctrlKey: true }), bigger)).toBe(true);
    expect(matchesShortcut(press('Equal', { ctrlKey: true, shiftKey: true }), bigger)).toBe(true);
    expect(matchesShortcut(press('NumpadAdd', { ctrlKey: true }), bigger)).toBe(true);
    expect(matchesShortcut(press('Minus', { ctrlKey: true }), SHORTCUTS.textSmaller)).toBe(true);
    expect(matchesShortcut(press('NumpadSubtract', { ctrlKey: true }), SHORTCUTS.textSmaller)).toBe(
      true,
    );
    expect(matchesShortcut(press('Digit0', { ctrlKey: true }), SHORTCUTS.textNormal)).toBe(true);
    expect(matchesShortcut(press('Numpad0', { ctrlKey: true }), SHORTCUTS.textNormal)).toBe(true);
    // The + key alone is the date key, not the text size key.
    expect(matchesShortcut(press('Equal'), bigger)).toBe(false);
    expect(matchesShortcut(press('Equal', { shiftKey: true }), SHORTCUTS.dateNextDay)).toBe(true);
  });
});

describe('menu side', () => {
  it('the menu is on the start side, and the way into the screen is the other side', () => {
    expect(menuSide('rtl')).toBe('right');
    expect(intoScreenSide('rtl')).toBe('left');
    expect(menuSide('ltr')).toBe('left');
    expect(intoScreenSide('ltr')).toBe('right');
  });
});

describe('Esc dispatcher', () => {
  it('sends the Esc key to the handler chosen by the order, and keeps other keys and shortcuts out', () => {
    const calls: string[] = [];
    const remove = registerEscape('leave', () => calls.push('leave'));
    const Mount = () => {
      useEscapeDispatcher();
      return null;
    };
    render(<Mount />);
    expect(fireEvent.keyDown(window, { key: 'Escape' })).toBe(false);
    fireEvent.keyDown(window, { key: 'Escape', ctrlKey: true });
    fireEvent.keyDown(window, { key: 'Enter' });
    expect(calls).toEqual(['leave']);
    remove();
    // With nothing to answer, the key is left alone.
    expect(fireEvent.keyDown(window, { key: 'Escape' })).toBe(true);
  });
});

describe('Esc order (PRD FR-7.18)', () => {
  const removers: (() => void)[] = [];
  afterEach(() => {
    for (const remove of removers.splice(0)) remove();
  });
  const add = (
    level: Parameters<typeof registerEscape>[0],
    handler: () => void,
    isActive?: () => boolean,
  ) => {
    removers.push(registerEscape(level, handler, isActive));
  };

  it('does nothing when nothing answers Esc', () => {
    expect(runEscape()).toBe(false);
  });

  it('closes a list or dialog first, then cancels typing, then leaves the screen', () => {
    const calls: string[] = [];
    add('leave', () => calls.push('leave'));
    add('edit', () => calls.push('edit'));
    add('overlay', () => calls.push('overlay'));
    // Registered last is the lowest level, yet the overlay still answers first.
    expect(runEscape()).toBe(true);
    expect(calls).toEqual(['overlay']);
  });

  it('answers with the next level when the higher one is gone', () => {
    const calls: string[] = [];
    add('leave', () => calls.push('leave'));
    const removeEdit = registerEscape('edit', () => calls.push('edit'));
    runEscape();
    removeEdit();
    runEscape();
    expect(calls).toEqual(['edit', 'leave']);
  });

  it('inside one level, the one opened last answers first (a list inside a dialog)', () => {
    const calls: string[] = [];
    add('overlay', () => calls.push('dialog'));
    add('overlay', () => calls.push('list'));
    runEscape();
    expect(calls).toEqual(['list']);
  });

  it('skips a handler that says it is not active at the moment Esc is pressed', () => {
    const calls: string[] = [];
    let editing = false;
    add('leave', () => calls.push('leave'));
    add(
      'edit',
      () => calls.push('edit'),
      () => editing,
    );
    runEscape();
    editing = true;
    runEscape();
    expect(calls).toEqual(['leave', 'edit']);
  });

  it('a removed handler no longer answers', () => {
    const handler = vi.fn();
    const remove = registerEscape('leave', handler);
    remove();
    expect(runEscape()).toBe(false);
    expect(handler).not.toHaveBeenCalled();
  });
});
