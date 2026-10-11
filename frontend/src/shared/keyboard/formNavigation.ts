// Arrow keys and Enter in forms (PRD FR-7.13, FR-7.15, FR-7.16, FR-7.22). Put the props on the
// element that holds the fields:
//   <form {...formNavigationProps}> ... </form>
// ↑ ↓ ← → move to the field the user sees in that direction. ← and → leave a text field only
// when its whole value is selected or the cursor is already at the edge (shouldLeaveField).
// Enter in a text field moves to the next field in reading order. While a list is open
// (aria-expanded="true", as pickers set it) the list owns ↑ ↓ and Enter. Entering a field
// selects its whole value. Screens never write their own arrow handlers (Code Standards F27).

import { useMemo, type FocusEvent, type KeyboardEvent } from 'react';
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

function makeKeyDown(onSubmit: (() => void) | undefined) {
  return (event: KeyboardEvent<HTMLElement>): void => handleKeyDown(event, onSubmit);
}

function handleKeyDown(event: KeyboardEvent<HTMLElement>, onSubmit: (() => void) | undefined): void {
  if (event.defaultPrevented || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) {
    return;
  }
  const target = event.target;
  if (!(target instanceof HTMLElement)) return;
  // A list that is open (a picker, a select) owns ↑ ↓ Enter and Esc (FR-7.16).
  if (target.getAttribute('aria-expanded') === 'true' || target.closest('[data-nav-owns-keys]')) {
    return;
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
      if (onSubmit !== undefined) {
        event.preventDefault();
        onSubmit();
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

function onFocus(event: FocusEvent<HTMLElement>): void {
  const target = event.target;
  if (target instanceof HTMLElement && isTextField(target) && !target.readOnly) target.select();
}

/** Put these on the element that holds a form's fields. */
export const formNavigationProps = { onKeyDown: makeKeyDown(undefined), onFocus } as const;

/**
 * The same, and Enter in the last field runs `onSubmit` (the sign-in screen: "Enter also logs
 * in").
 */
export function useFormNavigation(onSubmit: () => void) {
  return useMemo(() => ({ onKeyDown: makeKeyDown(onSubmit), onFocus }), [onSubmit]);
}
