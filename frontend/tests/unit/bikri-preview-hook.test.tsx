// The live preview waits 50 ms after the last key and asks once (NFR-SPD-7, Code Standards F35).
// Tested on the hook with fake timers, because a screen test cannot measure 50 ms fairly.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, renderHook } from '@testing-library/react';
import type { ReactNode } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as bikriApi from '../../src/features/bikri/api';
import { createBikriDraft } from '../../src/features/bikri/draft-store';
import { PREVIEW_DELAY_MS, useBikriPreview } from '../../src/features/bikri/hooks/useBikriPreview';
import { EVENT1 } from '../helpers/bikri-data';

vi.mock('../../src/features/bikri/api');

function setup() {
  const stores = createBikriDraft('10-10-2026');
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
  const hook = renderHook(() => useBikriPreview(stores), { wrapper });
  const type = (truckNo: string) => {
    act(() => {
      stores.header.getState().patch({ truckNo });
    });
  };
  return { stores, hook, type };
}

const wait = (ms: number) => act(() => vi.advanceTimersByTimeAsync(ms));

describe('the live preview timing', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.mocked(bikriApi.bikriPreview).mockReset();
    vi.mocked(bikriApi.bikriPreview).mockResolvedValue(EVENT1);
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('is 50 ms', () => {
    expect(PREVIEW_DELAY_MS).toBe(50);
  });

  it('asks nothing for a screen nobody has typed on', async () => {
    setup();
    await wait(500);
    expect(bikriApi.bikriPreview).not.toHaveBeenCalled();
  });

  it('asks once, 50 ms after the key and not before', async () => {
    const { type } = setup();
    type('L');
    await wait(PREVIEW_DELAY_MS - 1);
    expect(bikriApi.bikriPreview).not.toHaveBeenCalled();
    await wait(1);
    expect(bikriApi.bikriPreview).toHaveBeenCalledTimes(1);
  });

  it('keys typed closer than 50 ms apart ask once, for the last draft only', async () => {
    const { type } = setup();
    type('L');
    await wait(30);
    type('LE');
    await wait(30);
    type('LES');
    await wait(PREVIEW_DELAY_MS - 1);
    expect(bikriApi.bikriPreview).not.toHaveBeenCalled();
    await wait(1);
    expect(bikriApi.bikriPreview).toHaveBeenCalledTimes(1);
    expect(vi.mocked(bikriApi.bikriPreview).mock.calls[0]?.[0].header.truck_no).toBe('LES');
  });
});
