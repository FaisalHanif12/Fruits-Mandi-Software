// The step lines of a running question (PRD FR-7.73) come over an event channel. If that channel
// cannot be listened to, the panel must still work and nothing may be left as an unhandled error:
// the full run found one for every test that opened the shell (D8 end gate). Vitest fails a run on an
// unhandled rejection, so this test fails if the listener's failure is not caught.

import { renderHook } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import * as assistantApi from '../../src/features/assistant/api';
import { useStepLines } from '../../src/features/assistant/hooks';

vi.mock('../../src/features/assistant/api');

beforeEach(() => {
  vi.resetAllMocks();
});

/** Lets the promise of the failed listener settle before the test ends. */
const settle = () =>
  new Promise<void>((resolve) => {
    setTimeout(resolve, 20);
  });

describe('the channel of the step lines', () => {
  it('a channel that cannot be listened to leaves the panel working and no unhandled error', async () => {
    vi.mocked(assistantApi.listenSteps).mockRejectedValue(new Error('no channel'));
    const { unmount } = renderHook(() => {
      useStepLines();
    });
    await settle();
    expect(assistantApi.listenSteps).toHaveBeenCalledTimes(1);
    expect(() => {
      unmount();
    }).not.toThrow();
  });

  it('a channel that answers is switched off again when the panel goes away', async () => {
    const off = vi.fn();
    vi.mocked(assistantApi.listenSteps).mockResolvedValue(off);
    const { unmount } = renderHook(() => {
      useStepLines();
    });
    await settle();
    unmount();
    expect(off).toHaveBeenCalledTimes(1);
  });
});
