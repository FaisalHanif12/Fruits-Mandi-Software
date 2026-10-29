// The period a list is opened on (plan 38 R4, PRD FR-19.2, FR-7.45): a default for the kind of list
// (today, this month, this year), the Owner's last choice kept for the screen, and the two dates when
// he types his own. `today` is the day the app is on (the day status), as YYYY-MM-DD; until it is
// known the list waits (`ready`). Nothing here is money.

import { useCallback, useMemo, useState } from 'react';
import { PERIOD_KINDS, periodRange, type PeriodKind, type PeriodRange } from '../format';
import { useScreenMemory } from '../screen-state/hooks';
import { choiceOr, isRecord, textOr } from '../screen-state/remember';

/** The period of a screen. `from` and `to` count only when `kind` is `custom`. */
export interface PeriodState {
  kind: PeriodKind;
  from: string;
  to: string;
}

const MEMORY_PREFIX = 'period.';

function stateFrom(saved: unknown, fallback: PeriodKind): PeriodState {
  if (!isRecord(saved)) return { kind: fallback, from: '', to: '' };
  return {
    kind: choiceOr(saved['kind'], PERIOD_KINDS, fallback),
    from: textOr(saved['from'], ''),
    to: textOr(saved['to'], ''),
  };
}

export interface PeriodControl {
  state: PeriodState;
  /** The days to ask the backend for, as DD-MM-YYYY (an empty end is open). */
  range: PeriodRange;
  /** The saved choice has arrived and the day is known: the list can ask. */
  ready: boolean;
  /** Chooses another period; the choice is kept for the screen. */
  change: (next: PeriodState) => void;
}

/**
 * `screen` names the list (`bikri.today`: lower-case letters, digits and `_ - .`), `fallback` is
 * the period it opens on the first time, and `todayIso` is the day the app is on.
 */
export function usePeriod(
  screen: string,
  fallback: PeriodKind,
  todayIso: string | undefined,
): PeriodControl {
  const memory = useScreenMemory(MEMORY_PREFIX + screen);
  const saved = useMemo(() => stateFrom(memory.saved, fallback), [memory.saved, fallback]);
  const [chosen, setChosen] = useState<PeriodState | null>(null);
  const state = chosen ?? saved;
  const range = useMemo(
    () =>
      state.kind === 'custom'
        ? { from: state.from, to: state.to }
        : periodRange(state.kind, todayIso ?? ''),
    [state, todayIso],
  );
  const { remember } = memory;
  const change = useCallback(
    (next: PeriodState) => {
      setChosen(next);
      remember(next.kind === 'custom' ? next : { kind: next.kind });
    },
    [remember],
  );
  const ready =
    memory.ready && (todayIso !== undefined || state.kind === 'custom' || state.kind === 'all');
  return { state, range, ready, change };
}
