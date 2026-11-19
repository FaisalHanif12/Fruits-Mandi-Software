// Lets the answer of a live preview reach the screen before a test presses F10. The screen asks the
// cash question first only when the answer it shows is for what is typed now (`current` in
// `usePreviewQuery`); TanStack Query hands an answer to the screen in a timer, not in a microtask,
// so on a busy machine "the call was made and its promise is done" is not yet "the screen has it".
// A test that waits for the last call and then calls this does not race.

import { act } from '@testing-library/react';

const SETTLE_MS = 100;

export async function settlePreview(): Promise<void> {
  await act(async () => {
    await new Promise((resolve) => setTimeout(resolve, SETTLE_MS));
  });
}
