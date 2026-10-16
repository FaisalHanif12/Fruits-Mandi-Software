import { useCallback, useLayoutEffect, useRef } from 'react';

/**
 * A function that always gives the newest `value`, for code that runs later than the render that
 * made it: a key handler, a cell renderer, a callback a grid keeps. Such code must not capture the
 * value of one render, or it would read an old answer; and the screen must not read a ref while it
 * draws. The ref stays inside this hook, and the value is updated after each render.
 *
 *   const latest = useLatest(calc);
 *   const cellText = (row) => latest()?.[row.id];   // run by the grid when it draws a cell
 */
export function useLatest<T>(value: T): () => T {
  const ref = useRef(value);
  useLayoutEffect(() => {
    ref.current = value;
  });
  return useCallback(() => ref.current, []);
}
