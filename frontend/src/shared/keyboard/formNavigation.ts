// Arrow keys and Enter in forms (PRD FR-7.13, FR-7.15, FR-7.16, FR-7.22). Put the props on the
// element that holds the fields:
//   <form {...formNavigationProps}> ... </form>
// ↑ ↓ ← → move to the field the user sees in that direction. ← and → leave a text field only
// when its whole value is selected or the cursor is already at the edge (shouldLeaveField).
// Enter in a text field moves to the next field in reading order. While a list is open
// (aria-expanded="true", as pickers set it) the list owns ↑ ↓ and Enter; a picker's ← and →
// still leave the field like in any text field (data-picker-field). Entering a field
// selects its whole value. Screens never write their own arrow handlers (Code Standards F27).

import {
  useCallback,
  useLayoutEffect,
  useMemo,
  useRef,
  type FocusEvent,
  type KeyboardEvent,
} from 'react';
import { arrowSide, elementDirection, menuSide } from './direction';
import { focusRegion } from './focusRegions';
import { pickNeighbour, shouldLeaveField, type Move } from './fieldNavigation';

const FIELD_SELECTOR =
  'input:not([type="hidden"]), textarea, select, [role="combobox"], [data-nav-field]';
const TEXT_TYPES = new Set(['text', 'search', 'tel', 'email', 'url', 'password', '']);

type TextField = HTMLInputElement | HTMLTextAreaElement;

function isTextField(element: HTMLElement): element is TextField {
  if (element instanceof HTMLTextAreaElement) return true;
  return element instanceof HTMLInputElement && TEXT_TYPES.has(element.type);
}

function fieldsOf(container: HTMLElement): HTMLElement[] {
  return Array.from(container.querySelectorAll<HTMLElement>(FIELD_SELECTOR)).filter(
    (field) => !field.hasAttribute('disabled') && !field.hasAttribute('data-nav-skip'),
  );
}

function moveOf(key: string): Move | null {
  switch (key) {
    case 'ArrowUp':
      return 'up';
    case 'ArrowDown':
      return 'down';
    case 'ArrowLeft':
      return 'left';
    case 'ArrowRight':
      return 'right';
    default:
      return null;
  }
}

function focusField(field: HTMLElement): void {
  field.focus();
  if (isTextField(field)) field.select();
}

// Enter keys that a picker has already used (it picked, or kept the record it has) and that found
// no field after it. The form treats them as an Enter on the last field, so it can finish (plan
// 46): a picker's list is open whenever the field has the focus, and an open list keeps Enter.
const passedOn = new WeakSet<Event>();

/** Tells the form that this Enter was used by a picker that has no field after it to go to. */
export function passEnterToForm(event: KeyboardEvent<HTMLElement>): void {
  passedOn.add(event.nativeEvent);
}

/** How a form finishes: `now` for an Enter the form got itself, `afterPick` for a picker's Enter. */
interface Finish {
  now: () => void;
  afterPick: () => void;
}

function handleKeyDown(event: KeyboardEvent<HTMLElement>, finish: Finish | undefined): void {
  const passed = event.key === 'Enter' && passedOn.has(event.nativeEvent);
  if (
    (event.defaultPrevented && !passed) ||
    event.altKey ||
    event.ctrlKey ||
    event.metaKey ||
    event.shiftKey
  ) {
    return;
  }
  const target = event.target;
  if (!(target instanceof HTMLElement)) return;
  // A list that is open (a picker, a select) owns ↑ ↓ Enter and Esc (FR-7.16). A picker's list is
  // open whenever the field has the focus, so ← and → still leave a picker like any text field
  // (FR-7.15, plan 46); the open list of anything else keeps every key.
  if (target.closest('[data-nav-owns-keys]')) return;
  if (target.getAttribute('aria-expanded') === 'true' && !passed) {
    const sideways = event.key === 'ArrowLeft' || event.key === 'ArrowRight';
    if (!(sideways && target.hasAttribute('data-picker-field'))) return;
  }
  const container = event.currentTarget;
  const fields = fieldsOf(container);
  const from = fields.indexOf(target);
  if (from === -1) return;

  if (event.key === 'Enter') {
    if (!isTextField(target) || target instanceof HTMLTextAreaElement) return;
    const next = fields.slice(from + 1).find((field) => !field.matches('[readonly]'));
    if (next === undefined) {
      // The last field: Enter finishes the form, when the screen says what finishing means.
      if (finish !== undefined) {
        event.preventDefault();
        if (passed) finish.afterPick();
        else finish.now();
      }
      return;
    }
    focusField(next);
    event.preventDefault();
    return;
  }

  const move = moveOf(event.key);
  if (move === null) return;
  if (target instanceof HTMLTextAreaElement && (move === 'up' || move === 'down')) return;
  const side = arrowSide(event.key);
  const direction = elementDirection(target);
  if (side !== null && isTextField(target) && !shouldLeaveField(target, side, direction)) return;

  const to = pickNeighbour(
    fields.map((field) => field.getBoundingClientRect()),
    from,
    move,
  );
  const neighbour = to === null ? undefined : fields[to];
  if (neighbour !== undefined) {
    focusField(neighbour);
    event.preventDefault();
    return;
  }
  // No field that way. At the menu's edge of the screen the arrow goes back to the menu (FR-7.17).
  if (side !== null && side === menuSide(elementDirection(container)) && focusRegion('menu')) {
    event.preventDefault();
  }
}

/**
 * Enter on the last field of a form that finishes with Enter, when that field is a Select (a Radix trigger, plan 95): with its
 * list CLOSED the form finishes, as Enter in a last text field does (the short form of an item ends with "sold by", a Select).
 * With the list open Enter only picks the option, which the list owns. It is asked in the capture phase, before the trigger
 * opens its list on Enter, and a Select that is not the last field keeps its own Enter (it opens).
 */
function makeKeyDownCapture(onSubmit: (() => void) | undefined) {
  return (event: KeyboardEvent<HTMLElement>): void => {
    if (onSubmit === undefined || event.key !== 'Enter') return;
    if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
    const target = event.target;
    if (!(target instanceof HTMLButtonElement) || target.getAttribute('role') !== 'combobox')
      return;
    if (target.getAttribute('aria-expanded') === 'true' || target.closest('[data-nav-owns-keys]')) {
      return;
    }
    const fields = fieldsOf(event.currentTarget);
    const from = fields.indexOf(target);
    if (from === -1) return;
    // A Radix Select keeps a hidden native <select> beside its trigger; it is not a field the user can go to.
    if (
      fields.slice(from + 1).some((field) => !field.matches('[readonly], [aria-hidden="true"]'))
    ) {
      return;
    }
    event.preventDefault();
    onSubmit();
  };
}

function onFocus(event: FocusEvent<HTMLElement>): void {
  const target = event.target;
  if (target instanceof HTMLElement && isTextField(target) && !target.readOnly) target.select();
}

/**
 * Moves focus to the next field of the form around `from`, selecting its value, as Enter does.
 * A field that takes Enter for itself (the Urdu typing field turning a word into Urdu) calls this
 * after it has done its own work. False when there is no next field.
 */
export function focusNextField(from: HTMLElement): boolean {
  const container = from.closest<HTMLElement>('[data-form-zone]');
  if (container === null) return false;
  const fields = fieldsOf(container);
  const next = fields.slice(fields.indexOf(from) + 1).find((field) => !field.matches('[readonly]'));
  if (next === undefined) return false;
  focusField(next);
  return true;
}

/** Put these on the element that holds a form's fields. */
export const formNavigationProps = {
  onKeyDown: (event: KeyboardEvent<HTMLElement>): void => {
    handleKeyDown(event, undefined);
  },
  onFocus,
  'data-form-zone': '',
} as const;

/**
 * The same, and Enter in the last field runs `onSubmit` (the sign-in screen: "Enter also logs
 * in"). When a picker's Enter picks on the last field, the form finishes only after the screen has
 * the picked value: the `onSubmit` of this render still holds the values from before the pick, so
 * the newest one is run once the pick is drawn.
 */
export function useFormNavigation(onSubmit: () => void) {
  const latest = useRef(onSubmit);
  useLayoutEffect(() => {
    latest.current = onSubmit;
  });
  const onKeyDown = useCallback(
    (event: KeyboardEvent<HTMLElement>): void => {
      handleKeyDown(event, {
        now: onSubmit,
        afterPick: () => {
          // React draws the pick when this key event ends; the microtask runs after that.
          queueMicrotask(() => {
            latest.current();
          });
        },
      });
    },
    [onSubmit],
  );
  return useMemo(
    () => ({
      onKeyDown,
      onKeyDownCapture: makeKeyDownCapture(onSubmit),
      onFocus,
      'data-form-zone': '',
    }),
    [onKeyDown, onSubmit],
  );
}
