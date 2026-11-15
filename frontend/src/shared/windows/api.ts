// The command behind the window of a list (PRD FR-19.11, plan 76 point 8): the first day that has data, for the
// back button of its 7-day or 1-month window. Read only. Components never call invoke() (Code Standards F4).

import type { WindowsBounds } from '../../bindings/WindowsBounds';
import type { WindowsBoundsInput } from '../../bindings/WindowsBoundsInput';
import { callCommand } from '../api';

/** The first day that has data in a list, and today as the last day a window may show. */
export function windowsBounds(input: WindowsBoundsInput): Promise<WindowsBounds> {
  return callCommand<WindowsBounds>('windows_bounds', { input });
}
