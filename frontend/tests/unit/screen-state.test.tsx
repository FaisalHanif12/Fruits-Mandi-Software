// What a screen remembers for the Owner (round 9, PRD FR-7.45): the hook reads the saved text of a
// screen once, saves after a pause, saves at once when the screen closes, and does not write what it
// read back unchanged; `Remembered` draws the screen only when the saved text has arrived or could
// not be read. The reading helpers never throw on a text that is not what the screen wrote.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, renderHook, screen, waitFor } from '@testing-library/react';
import { createElement, type ReactNode } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import * as api from '../../src/shared/screen-state/api';
import {
  REMEMBER_SETTLE_MS,
  Remembered,
  canonicalText,
  choiceOr,
  idOrNull,
  pickedOrNull,
  pickedText,
  readRemembered,
  rememberedText,
  useScreenMemory,
} from '../../src/shared/screen-state';
import { CommandError } from '../../src/shared/api';
import { refusal } from '../helpers/ui-error';
import { renderIn } from '../helpers/render';

const get = vi.mocked(api.prefsScreenGet);
const set = vi.mocked(api.prefsScreenSet);

function wrapper() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return function Wrapper({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
  };
}

beforeEach(() => {
  get.mockClear();
  set.mockClear();
  get.mockImplementation(({ screen: name }) => Promise.resolve({ screen: name, value: null }));
  set.mockImplementation(({ screen: name, value }) =>
    Promise.resolve({ screen: name, value: value === '' ? null : value }),
  );
});

describe('the text a screen keeps', () => {
  it('writes a value as compact JSON, and nothing for null or for a text the backend would refuse', () => {
    expect(rememberedText({ city_id: 3 })).toBe('{"city_id":3}');
    expect(rememberedText(null)).toBe('');
    expect(rememberedText(undefined)).toBe('');
    expect(rememberedText('x'.repeat(5_000))).toBe('');
  });

  it('reads a text back, and reads nothing from an empty or a broken one', () => {
    expect(readRemembered('{"city_id":3}')).toEqual({ city_id: 3 });
    expect(readRemembered('')).toBeNull();
    expect(readRemembered(null)).toBeNull();
    expect(readRemembered('{not json')).toBeNull();
  });

  it('two texts of the same value are the same whatever order their keys come in', () => {
    expect(canonicalText('{"b":1,"a":{"d":2,"c":3}}')).toBe(
      canonicalText('{"a":{"c":3,"d":2},"b":1}'),
    );
    expect(canonicalText('')).toBe('');
  });

  it('takes only what has the right shape: a choice from a list, an id, a picked record', () => {
    expect(choiceOr('due', ['all', 'paid', 'due'], 'all')).toBe('due');
    expect(choiceOr('other', ['all', 'paid', 'due'], 'all')).toBe('all');
    expect(choiceOr(5, ['all'], 'all')).toBe('all');
    expect(idOrNull(7)).toBe(7);
    expect(idOrNull(0)).toBeNull();
    expect(idOrNull('7')).toBeNull();
    expect(idOrNull(1.5)).toBeNull();
    expect(pickedOrNull({ id: 4, name_ur: 'اکرم', name_en: 'Akram', code: '0020' })).toEqual({
      id: 4,
      name_ur: 'اکرم',
      name_en: 'Akram',
      code: '0020',
    });
    expect(pickedOrNull({ id: 'x' })).toBeNull();
    expect(pickedOrNull(null)).toBeNull();
    expect(pickedText({ id: 4, name_ur: 'اکرم', name_en: 'Akram' })).toEqual({
      id: 4,
      name_ur: 'اکرم',
      name_en: 'Akram',
    });
    expect(pickedText(null)).toBeNull();
  });
});

describe('useScreenMemory', () => {
  it('is ready with what the screen saved, parsed from its JSON', async () => {
    get.mockResolvedValue({ screen: 'report.r20', value: '{"city_id":3}' });
    const { result } = renderHook(() => useScreenMemory('report.r20'), { wrapper: wrapper() });
    expect(result.current.ready).toBe(false);
    await waitFor(() => {
      expect(result.current.ready).toBe(true);
    });
    expect(result.current.saved).toEqual({ city_id: 3 });
    expect(get).toHaveBeenCalledWith({ screen: 'report.r20' });
  });

  it('is ready with nothing when the saved text cannot be read at all (the screen starts as new)', async () => {
    get.mockRejectedValue(new CommandError(refusal([{ code: 'ERR_INTERNAL' }])));
    const { result } = renderHook(() => useScreenMemory('report.r20'), { wrapper: wrapper() });
    await waitFor(() => {
      expect(result.current.ready).toBe(true);
    });
    expect(result.current.saved).toBeNull();
  });

  it('saves a choice after a pause, once, with the text the screen wrote', async () => {
    const { result } = renderHook(() => useScreenMemory('report.r20'), { wrapper: wrapper() });
    await waitFor(() => {
      expect(result.current.ready).toBe(true);
    });
    act(() => {
      result.current.remember({ city_id: 3 });
      result.current.remember({ city_id: 4 });
    });
    expect(set).not.toHaveBeenCalled();
    await waitFor(
      () => {
        expect(set).toHaveBeenCalledTimes(1);
      },
      { timeout: REMEMBER_SETTLE_MS * 6 },
    );
    expect(set).toHaveBeenCalledWith({ screen: 'report.r20', value: '{"city_id":4}' });
  });

  it('forgets a screen when it remembers null, and writes nothing when the value did not change', async () => {
    get.mockResolvedValue({ screen: 'report.r20', value: '{"b":1,"a":2}' });
    const { result } = renderHook(() => useScreenMemory('report.r20'), { wrapper: wrapper() });
    await waitFor(() => {
      expect(result.current.ready).toBe(true);
    });
    // The same value, written with its keys in another order, is not a change.
    act(() => {
      result.current.remember({ a: 2, b: 1 });
    });
    await new Promise((resolve) => setTimeout(resolve, REMEMBER_SETTLE_MS + 200));
    expect(set).not.toHaveBeenCalled();
    act(() => {
      result.current.remember(null);
    });
    await waitFor(
      () => {
        expect(set).toHaveBeenCalledWith({ screen: 'report.r20', value: '' });
      },
      { timeout: REMEMBER_SETTLE_MS * 6 },
    );
  });

  it('saves at once when the screen closes, so leaving straight away loses nothing', async () => {
    const { result, unmount } = renderHook(() => useScreenMemory('list.bikri-stock'), {
      wrapper: wrapper(),
    });
    await waitFor(() => {
      expect(result.current.ready).toBe(true);
    });
    act(() => {
      result.current.remember({ includeWrittenOff: true });
    });
    unmount();
    expect(set).toHaveBeenCalledWith({
      screen: 'list.bikri-stock',
      value: '{"includeWrittenOff":true}',
    });
  });

  it('a failed save is never shown and never throws', async () => {
    set.mockRejectedValue(new CommandError(refusal([{ code: 'ERR_OUT_OF_RANGE' }])));
    const { result, unmount } = renderHook(() => useScreenMemory('report.r20'), {
      wrapper: wrapper(),
    });
    await waitFor(() => {
      expect(result.current.ready).toBe(true);
    });
    act(() => {
      result.current.remember({ x: 1 });
    });
    expect(() => {
      unmount();
    }).not.toThrow();
  });
});

/** What a screen shows of what it kept, in words. */
function label(saved: unknown): string {
  return saved === null ? 'new' : 'old';
}

/** The names of the screens under test. */
const PUKHTA_SCREEN = 'list.pukhta-register';
const REPORT_SCREEN = 'report.r20';

describe('Remembered', () => {
  it('shows nothing of the screen until the saved text has arrived, then gives it to the screen', async () => {
    get.mockResolvedValue({ screen: PUKHTA_SCREEN, value: '{"state":"due"}' });
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    renderIn(
      'en',
      <QueryClientProvider client={client}>
        {createElement(Remembered, {
          screen: PUKHTA_SCREEN,
          children: ({ saved }) => <p>{JSON.stringify(saved)}</p>,
        })}
      </QueryClientProvider>,
    );
    expect(screen.queryByText('{"state":"due"}')).not.toBeInTheDocument();
    expect(await screen.findByText('{"state":"due"}')).toBeInTheDocument();
  });

  it('draws the screen with nothing saved when the saved text cannot be read', async () => {
    get.mockRejectedValue(new Error('no bridge'));
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    renderIn(
      'en',
      <QueryClientProvider client={client}>
        {createElement(Remembered, {
          screen: REPORT_SCREEN,
          children: ({ saved }) => createElement('p', null, label(saved)),
        })}
      </QueryClientProvider>,
    );
    expect(await screen.findByText('new')).toBeInTheDocument();
  });
});
