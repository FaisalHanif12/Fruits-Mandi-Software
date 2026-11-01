// A focus trap for a panel that stays open beside the screen (the question panel, plan 49, PRD
// FR-7.73): while focus is inside it, Tab and Shift+Tab go round its own controls and never leave it.
// It does not pull focus back when the user clicks outside: following a link from an answer must be
// able to put focus on the screen it opened. Esc and F1 are the ways out by keyboard (FR-7.18).

import type { KeyboardEvent as ReactKeyboardEvent } from 'react';

/** Everything in a panel that Tab can reach. */
const TABBABLE =
  'input:not([type="hidden"]):not([disabled]), select:not([disabled]), textarea:not([disabled]), button:not([disabled]), a[href], [role="combobox"], [tabindex]:not([tabindex="-1"])';

/** The controls of `container` that Tab reaches, in page order, leaving out any that are hidden. */
export function tabbableIn(container: HTMLElement): HTMLElement[] {
  return Array.from(container.querySelectorAll<HTMLElement>(TABBABLE)).filter(
    (element) => element.closest('[inert], [hidden], [aria-hidden="true"]') === null,
  );
}

/**
 * Keeps Tab inside `container`. Call it from the container's key handler. True when it moved the
 * focus (the event is then stopped).
 */
export function trapTab(event: ReactKeyboardEvent<HTMLElement>, container: HTMLElement): boolean {
  if (event.key !== 'Tab' || event.ctrlKey || event.altKey || event.metaKey) return false;
  const controls = tabbableIn(container);
  const first = controls.at(0);
  const last = controls.at(-1);
  if (first === undefined || last === undefined) return false;
  const active = document.activeElement;
  if (event.shiftKey && (active === first || !container.contains(active))) {
    event.preventDefault();
    last.focus();
    return true;
  }
  if (!event.shiftKey && (active === last || !container.contains(active))) {
    event.preventDefault();
    first.focus();
    return true;
  }
  return false;
}
