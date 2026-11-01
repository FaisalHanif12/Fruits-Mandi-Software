// The reason typed for an edit, kept in a ref (plan 44): the save reads the reason of the box that
// asked for it, even though the box closes before the save is sent. Same as the sale book's.

import { useMemo, useRef } from 'react';

export interface ReasonText {
  get: () => string;
  set: (text: string) => void;
}

export function useReasonText(initial = ''): ReasonText {
  const text = useRef(initial);
  return useMemo(
    () => ({
      get: () => text.current,
      set: (next) => {
        text.current = next;
      },
    }),
    [],
  );
}
