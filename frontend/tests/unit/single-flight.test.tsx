// The one-save-at-a-time guard (NFR-REL-1): a second start while a task is running does nothing,
// also when both come in the same instant, and the guard opens again when the task is over,
// whether it worked or failed.

import { act, renderHook } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { useSingleFlight } from '../../src/shared/hooks';

/** A promise the test settles by hand. */
function later<T>() {
  let resolve: (value: T) => void = () => undefined;
  let reject: (reason: Error) => void = () => undefined;
  const promise = new Promise<T>((done, fail) => {
    resolve = done;
    reject = fail;
  });
  return { promise, resolve, reject };
}

describe('useSingleFlight', () => {
  it('starts the first task and answers the later ones with the busy answer, without starting them', async () => {
    const { result } = renderHook(() => useSingleFlight());
    const first = later<string>();
    const task = vi.fn(() => first.promise);
    const other = vi.fn(() => Promise.resolve('other'));
    let started: Promise<string> = Promise.resolve('');
    let second: Promise<string> = Promise.resolve('');
    let third: Promise<string> = Promise.resolve('');
    // Three starts inside one act(): React cannot redraw between them.
    act(() => {
      started = result.current.run(task, 'busy');
      second = result.current.run(other, 'busy');
      third = result.current.run(other, 'busy');
    });
    expect(task).toHaveBeenCalledTimes(1);
    expect(other).not.toHaveBeenCalled();
    expect(result.current.busy).toBe(true);
    expect(await second).toBe('busy');
    expect(await third).toBe('busy');
    await act(async () => {
      first.resolve('saved');
      await started;
    });
    expect(await started).toBe('saved');
    expect(result.current.busy).toBe(false);
  });

  it('opens again after the task is over', async () => {
    const { result } = renderHook(() => useSingleFlight());
    await act(async () => {
      await result.current.run(() => Promise.resolve(1), 0);
    });
    const task = vi.fn(() => Promise.resolve(2));
    let answer = 0;
    await act(async () => {
      answer = await result.current.run(task, 0);
    });
    expect(task).toHaveBeenCalledTimes(1);
    expect(answer).toBe(2);
  });

  it('opens again after a task that failed, and passes the failure on', async () => {
    const { result } = renderHook(() => useSingleFlight());
    const failing = later<string>();
    let attempt: Promise<string> = Promise.resolve('');
    act(() => {
      attempt = result.current.run(() => failing.promise, 'busy');
    });
    const caught = attempt.catch((error: unknown) => error);
    await act(async () => {
      failing.reject(new Error('disk full'));
      await caught;
    });
    expect(String(await caught)).toContain('disk full');
    expect(result.current.busy).toBe(false);
    let again = '';
    await act(async () => {
      again = await result.current.run(() => Promise.resolve('ok'), 'busy');
    });
    expect(again).toBe('ok');
  });

  it('opens again when the task throws before it returns a promise', async () => {
    const { result } = renderHook(() => useSingleFlight());
    const broken = (): Promise<string> => {
      throw new Error('broken');
    };
    let caught: unknown = null;
    act(() => {
      result.current.run(broken, 'busy').catch((error: unknown) => {
        caught = error;
      });
    });
    await act(async () => {
      await Promise.resolve();
    });
    expect(String(caught)).toContain('broken');
    expect(result.current.busy).toBe(false);
  });
});
