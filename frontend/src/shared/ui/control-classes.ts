// The look of every typed control (design system 6): 44 px, 4 px radius, control border.
// Error: red border on a red-soft ground. Read-only: the page ground. Shared by TextInput,
// the number inputs and Select, so they can never drift apart (Code Standards F12).

export interface ControlState {
  invalid?: boolean | undefined;
  readOnly?: boolean | undefined;
  /** A fully rounded box with more room at its sides: the question box of the chat window (plan 51). */
  pill?: boolean | undefined;
}

function stateClasses({ invalid = false, readOnly = false }: ControlState): string {
  if (invalid) return 'border-due bg-due-soft';
  if (readOnly) return 'border-border-control bg-bg';
  return 'border-border-control bg-surface';
}

export function controlClasses(state: ControlState): string {
  // The pill is the one new look; every other control keeps the exact classes it always had.
  if (state.pill === true) {
    return `h-control w-full min-w-0 rounded-full border px-5 text-ink transition-calm placeholder:text-ink-placeholder ${stateClasses(state)}`;
  }
  return `h-control w-full min-w-0 rounded-1 border px-3 text-ink transition-calm placeholder:text-ink-placeholder ${stateClasses(state)}`;
}
