// The reason typed for an edit (plan 16): kept outside the screens so the save reads the reason
// that was typed in the box that asked for it, even though the box closes before the save is sent.

import { useMemo, useRef } from 'react';

export interface EditReason {
  get: () => string;
  set: (text: string) => void;
}

export function useEditReason(): EditReason {
  const text = useRef('');
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
