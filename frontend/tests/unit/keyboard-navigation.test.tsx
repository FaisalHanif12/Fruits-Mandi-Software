// Arrow keys in forms, menus and lists (PRD FR-7.13 to FR-7.17), in both directions. jsdom has
// no layout, so each test gives the fields the boxes they would have on screen: in right-to-left
// the first field in reading order is on the right.

import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useEffect, useRef } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  formNavigationProps,
  intoScreenSide,
  moveIndex,
  pickNeighbour,
  registerFocusRegion,
  shouldLeaveField,
  useRovingKeys,
  type Box,
} from '../../src/shared/keyboard';

type Dir = 'rtl' | 'ltr';
const DIRECTIONS: readonly Dir[] = ['rtl', 'ltr'];

const WIDTH = 100;
const GAP = 20;
const ROW = 60;
const HEIGHT = 40;

/** The box of a field in column `col` (0 is the first in reading order) and row `row`. */
function boxAt(dir: Dir, col: number, row: number, columns = 2): Box {
  const slot = dir === 'ltr' ? col : columns - 1 - col;
  const left = slot * (WIDTH + GAP);
  return { left, right: left + WIDTH, top: row * ROW, bottom: row * ROW + HEIGHT };
}

describe('pickNeighbour', () => {
  describe.each(DIRECTIONS)('a two by two layout (%s)', (dir) => {
    // Reading order: a b / c d.
    const boxes = [boxAt(dir, 0, 0), boxAt(dir, 1, 0), boxAt(dir, 0, 1), boxAt(dir, 1, 1)];
    // Physical neighbours: in rtl, a is on the right of b.
    const toTheLeftOfA = dir === 'rtl' ? 1 : null;
    const toTheRightOfA = dir === 'rtl' ? null : 1;

    it('goes to the field the user sees in each direction', () => {
      expect(pickNeighbour(boxes, 0, 'left')).toBe(toTheLeftOfA);
      expect(pickNeighbour(boxes, 0, 'right')).toBe(toTheRightOfA);
      expect(pickNeighbour(boxes, 0, 'down')).toBe(2);
      expect(pickNeighbour(boxes, 0, 'up')).toBeNull();
      expect(pickNeighbour(boxes, 3, 'up')).toBe(1);
      expect(pickNeighbour(boxes, 3, 'down')).toBeNull();
    });

    it('stays in its row for left and right', () => {
      expect(pickNeighbour(boxes, 2, dir === 'rtl' ? 'left' : 'right')).toBe(3);
      expect(pickNeighbour(boxes, 2, dir === 'rtl' ? 'right' : 'left')).toBeNull();
    });
  });

  it('goes to the nearest row, and in it to the closest field across', () => {
    const wide: Box = { left: 0, right: 220, top: 0, bottom: 40 };
    const boxes = [
      wide,
      { left: 0, right: 100, top: 60, bottom: 100 },
      { left: 120, right: 220, top: 60, bottom: 100 },
      { left: 0, right: 100, top: 120, bottom: 160 },
    ];
    // From the wide field, the centre is 110: both fields below are 60 away, the first wins.
    expect(pickNeighbour(boxes, 0, 'down')).toBe(1);
    expect(pickNeighbour(boxes, 2, 'down')).toBe(3);
    expect(pickNeighbour(boxes, 3, 'down')).toBeNull();
    expect(pickNeighbour(boxes, 3, 'up')).toBe(1);
  });

  it('has no neighbour for an unknown field', () => {
    expect(pickNeighbour([], 0, 'down')).toBeNull();
  });
});

describe('shouldLeaveField (PRD FR-7.15)', () => {
  const caret = (value: string, start: number, end = start) => ({
    value,
    selectionStart: start,
    selectionEnd: end,
  });

  it.each(DIRECTIONS)('leaves while the whole value is selected, in %s', (dir) => {
    expect(shouldLeaveField(caret('abc', 0, 3), 'left', dir)).toBe(true);
    expect(shouldLeaveField(caret('abc', 0, 3), 'right', dir)).toBe(true);
  });

  it.each(DIRECTIONS)('leaves an empty field, in %s', (dir) => {
    expect(shouldLeaveField(caret('', 0), 'left', dir)).toBe(true);
    expect(shouldLeaveField(caret('', 0), 'right', dir)).toBe(true);
  });

  it('left to right: the left edge is the start of the text, the right edge its end', () => {
    expect(shouldLeaveField(caret('abc', 0), 'left', 'ltr')).toBe(true);
    expect(shouldLeaveField(caret('abc', 1), 'left', 'ltr')).toBe(false);
    expect(shouldLeaveField(caret('abc', 3), 'right', 'ltr')).toBe(true);
    expect(shouldLeaveField(caret('abc', 2), 'right', 'ltr')).toBe(false);
    expect(shouldLeaveField(caret('abc', 3), 'left', 'ltr')).toBe(false);
  });

  it('right to left: the right edge is the start of the text, the left edge its end', () => {
    expect(shouldLeaveField(caret('abc', 3), 'left', 'rtl')).toBe(true);
    expect(shouldLeaveField(caret('abc', 2), 'left', 'rtl')).toBe(false);
    expect(shouldLeaveField(caret('abc', 0), 'right', 'rtl')).toBe(true);
    expect(shouldLeaveField(caret('abc', 1), 'right', 'rtl')).toBe(false);
    expect(shouldLeaveField(caret('abc', 0), 'left', 'rtl')).toBe(false);
  });

  it('moves the cursor, not the field, when part of the text is selected', () => {
    expect(shouldLeaveField(caret('abc', 1, 2), 'left', 'ltr')).toBe(false);
    expect(shouldLeaveField(caret('abc', 1, 2), 'right', 'rtl')).toBe(false);
  });
});

/** Four fields, a b / c d in reading order, with their boxes for the direction. */
function Form({ dir, values = ['one', 'two', 'three', 'four'] }: { dir: Dir; values?: string[] }) {
  const ref = useRef<HTMLFormElement>(null);
  useEffect(() => {
    const fields = ref.current?.querySelectorAll('input') ?? [];
    fields.forEach((field, index) => {
      const box = boxAt(dir, index % 2, Math.floor(index / 2));
      field.getBoundingClientRect = () => ({
        ...box,
        width: WIDTH,
        height: HEIGHT,
        x: box.left,
        y: box.top,
        toJSON: () => box,
      });
    });
  }, [dir]);
  return (
    <form ref={ref} dir={dir} {...formNavigationProps}>
      {['a', 'b', 'c', 'd'].map((name, index) => (
        <input key={name} data-testid={name} defaultValue={values[index]} />
      ))}
    </form>
  );
}

const user = () => userEvent.setup();

describe.each(DIRECTIONS)('arrow keys in a form (%s)', (dir) => {
  // Where b is from a: to the left in rtl, to the right in ltr.
  const towardsB = dir === 'rtl' ? '{ArrowLeft}' : '{ArrowRight}';
  const awayFromB = dir === 'rtl' ? '{ArrowRight}' : '{ArrowLeft}';

  it('↓ and ↑ move between rows', async () => {
    render(<Form dir={dir} />);
    const u = user();
    screen.getByTestId('a').focus();
    await u.keyboard('{ArrowDown}');
    expect(screen.getByTestId('c')).toHaveFocus();
    await u.keyboard('{ArrowUp}');
    expect(screen.getByTestId('a')).toHaveFocus();
  });

  it('← and → move to the field on that side, once the value is selected (it is, on entering)', async () => {
    render(<Form dir={dir} />);
    const u = user();
    screen.getByTestId('a').focus();
    await u.keyboard(towardsB);
    expect(screen.getByTestId('b')).toHaveFocus();
    await u.keyboard(awayFromB);
    expect(screen.getByTestId('a')).toHaveFocus();
  });

  it('selects the whole value when a field is entered by an arrow', async () => {
    render(<Form dir={dir} />);
    screen.getByTestId('a').focus();
    await user().keyboard('{ArrowDown}');
    const c = screen.getByTestId<HTMLInputElement>('c');
    expect([c.selectionStart, c.selectionEnd]).toEqual([0, 5]);
  });

  it('moves the text cursor first once the user has moved it, and leaves at the edge', async () => {
    render(<Form dir={dir} />);
    const u = user();
    const a = screen.getByTestId<HTMLInputElement>('a');
    a.focus();
    // Put the cursor in the middle: now the arrows move it.
    a.setSelectionRange(1, 1);
    await u.keyboard(towardsB);
    expect(a).toHaveFocus();
    // Move the cursor to the edge that faces b, then the next press leaves.
    const atEdge = dir === 'rtl' ? a.value.length : a.value.length;
    a.setSelectionRange(atEdge, atEdge);
    // In ltr the edge that faces b (on the right) is the end; in rtl the edge that faces b (on
    // the left) is the end too. Both directions put b at the end of the text.
    await u.keyboard(towardsB);
    expect(screen.getByTestId('b')).toHaveFocus();
  });

  it('Enter moves to the next field in reading order and selects it', async () => {
    render(<Form dir={dir} />);
    const u = user();
    screen.getByTestId('a').focus();
    await u.keyboard('{Enter}');
    expect(screen.getByTestId('b')).toHaveFocus();
    await u.keyboard('{Enter}');
    expect(screen.getByTestId('c')).toHaveFocus();
    await u.keyboard('{Enter}{Enter}');
    expect(screen.getByTestId('d')).toHaveFocus();
  });

  it('at the edge of the screen on the menu side, the arrow goes back to the menu', async () => {
    const menu = document.createElement('nav');
    const item = document.createElement('button');
    menu.append(item);
    document.body.append(menu);
    const unregister = registerFocusRegion('menu', menu);
    render(<Form dir={dir} />);
    const u = user();
    screen.getByTestId('a').focus();
    // a is first in reading order, so it is on the start side: the side the menu is on.
    await u.keyboard(awayFromB);
    expect(item).toHaveFocus();
    unregister();
    menu.remove();
  });

  it('does nothing at the other edge', async () => {
    render(<Form dir={dir} />);
    screen.getByTestId('b').focus();
    await user().keyboard(towardsB);
    expect(screen.getByTestId('b')).toHaveFocus();
  });

  it('leaves ↑ ↓ and Enter to an open list', async () => {
    render(
      <form dir={dir} {...formNavigationProps}>
        <input data-testid="picker" aria-expanded="true" />
        <input data-testid="next" />
      </form>,
    );
    const u = user();
    screen.getByTestId('picker').focus();
    await u.keyboard('{ArrowDown}{Enter}');
    expect(screen.getByTestId('picker')).toHaveFocus();
  });

  it('ignores keys with Ctrl, Alt or Shift, so shortcuts keep working', async () => {
    render(<Form dir={dir} />);
    const u = user();
    screen.getByTestId('a').focus();
    await u.keyboard('{Alt>}{ArrowDown}{/Alt}');
    await u.keyboard('{Control>}{Enter}{/Control}');
    expect(screen.getByTestId('a')).toHaveFocus();
  });
});

describe('moveIndex', () => {
  it.each([
    [0, 'ArrowDown', 1],
    [3, 'ArrowDown', 4],
    [4, 'ArrowDown', 4],
    [0, 'ArrowUp', 0],
    [3, 'ArrowUp', 2],
    [1, 'PageDown', 4],
    [3, 'PageDown', 4],
    [4, 'PageUp', 1],
    [1, 'PageUp', 0],
    [2, 'Home', 0],
    [2, 'End', 4],
  ])('item %i with %s goes to %i of 5 (page of 3)', (from, key, to) => {
    expect(moveIndex(from, 5, key, 3)).toBe(to);
  });

  it('does not move on other keys or in an empty list', () => {
    expect(moveIndex(0, 5, 'a', 3)).toBeNull();
    expect(moveIndex(0, 0, 'ArrowDown', 3)).toBeNull();
  });
});

function Menu({
  dir,
  isMenu,
  onOpen,
}: {
  dir: Dir;
  isMenu: boolean;
  onOpen: (name: string) => void;
}) {
  const onKeyDown = useRovingKeys({ isMenu, pageSize: 2 });
  return (
    <div dir={dir} role="menu" tabIndex={-1} onKeyDown={onKeyDown}>
      <button
        data-nav-item
        data-testid="m1"
        onClick={() => {
          onOpen('m1');
        }}
      />
      <button data-nav-item data-testid="m2" disabled />
      <button
        data-nav-item
        data-testid="m3"
        onClick={() => {
          onOpen('m3');
        }}
      />
      <div
        data-nav-item
        tabIndex={0}
        data-testid="m4"
        role="menuitem"
        onClick={() => {
          onOpen('m4');
        }}
      />
      <div
        data-nav-item
        tabIndex={0}
        data-testid="m5"
        role="menuitem"
        onClick={() => {
          onOpen('m5');
        }}
      />
    </div>
  );
}

describe.each(DIRECTIONS)('arrow keys in a menu or list (%s)', (dir) => {
  const into = intoScreenSide(dir) === 'left' ? '{ArrowLeft}' : '{ArrowRight}';
  const back = intoScreenSide(dir) === 'left' ? '{ArrowRight}' : '{ArrowLeft}';

  afterEach(() => {
    document.body.innerHTML = '';
  });

  it('↑ ↓ move between items and skip a disabled one', async () => {
    render(<Menu dir={dir} isMenu onOpen={vi.fn()} />);
    const u = user();
    screen.getByTestId('m1').focus();
    await u.keyboard('{ArrowDown}');
    expect(screen.getByTestId('m3')).toHaveFocus();
    await u.keyboard('{ArrowDown}');
    expect(screen.getByTestId('m4')).toHaveFocus();
    await u.keyboard('{ArrowUp}{ArrowUp}');
    expect(screen.getByTestId('m1')).toHaveFocus();
  });

  it('Page Down, Page Up, Home and End move a page or to the ends', async () => {
    render(<Menu dir={dir} isMenu onOpen={vi.fn()} />);
    const u = user();
    screen.getByTestId('m1').focus();
    await u.keyboard('{PageDown}');
    expect(screen.getByTestId('m4')).toHaveFocus();
    await u.keyboard('{End}');
    expect(screen.getByTestId('m5')).toHaveFocus();
    // The enabled items are m1, m3, m4 and m5: a page is two of them.
    await u.keyboard('{PageUp}');
    expect(screen.getByTestId('m3')).toHaveFocus();
    await u.keyboard('{Home}');
    expect(screen.getByTestId('m1')).toHaveFocus();
  });

  it('Enter opens the focused item, once, whether it is a button or a row', async () => {
    const onOpen = vi.fn();
    render(<Menu dir={dir} isMenu onOpen={onOpen} />);
    const u = user();
    screen.getByTestId('m3').focus();
    await u.keyboard('{Enter}');
    screen.getByTestId('m4').focus();
    await u.keyboard('{Enter}');
    expect(onOpen.mock.calls).toEqual([['m3'], ['m4']]);
  });

  it('in a menu, the arrow into the screen moves focus to the screen, and the other does not', async () => {
    const screenArea = document.createElement('main');
    const field = document.createElement('input');
    screenArea.append(field);
    document.body.append(screenArea);
    const unregister = registerFocusRegion('screen', screenArea);
    render(<Menu dir={dir} isMenu onOpen={vi.fn()} />);
    const u = user();
    screen.getByTestId('m1').focus();
    await u.keyboard(back);
    expect(screen.getByTestId('m1')).toHaveFocus();
    await u.keyboard(into);
    expect(field).toHaveFocus();
    unregister();
  });

  it('a list that is not a menu leaves ← and → alone', async () => {
    render(<Menu dir={dir} isMenu={false} onOpen={vi.fn()} />);
    screen.getByTestId('m1').focus();
    await user().keyboard(into);
    expect(screen.getByTestId('m1')).toHaveFocus();
  });
});
