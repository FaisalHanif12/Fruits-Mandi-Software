// The draft is kept every 5 seconds while the user types (plan 13, PRD FR-12.10, NFR-REL-2): with
// fake timers, so five seconds pass in an instant. Nothing is sent for an empty draft; a draft the
// user emptied is thrown away; after a save no draft is sent, and one that was on its way is
// thrown away too.

import { render } from '@testing-library/react';
import { useEffect } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { useDraftAutosave } from '../../src/features/bikri/hooks/useDraftAutosave';
import { createBikriDraft, type BikriDraftStores } from '../../src/features/bikri/draft-store';
import { parseSnapshot } from '../../src/features/bikri/draft-snapshot';
import { LOT } from '../../src/features/bikri/types';
import * as draftsApi from '../../src/shared/drafts/api';
import { DRAFT_SAVE_INTERVAL_MS } from '../../src/shared/drafts';

vi.mock('../../src/shared/drafts/api');

/** What the test holds on to: the way to tell the hook the بکری was saved. */
const handle: { stop: () => void } = { stop: () => undefined };

function Harness({ stores }: { stores: BikriDraftStores }) {
  const { stop } = useDraftAutosave(stores, { enabled: true });
  useEffect(() => {
    handle.stop = stop;
  });
  return null;
}

const SAVED = { saved_at: '2026-10-10T07:30:00Z' };

beforeEach(() => {
  vi.useFakeTimers();
  vi.clearAllMocks();
  vi.mocked(draftsApi.draftsSave).mockResolvedValue(SAVED);
  vi.mocked(draftsApi.draftsDiscard).mockResolvedValue(null);
});

afterEach(() => {
  vi.useRealTimers();
});

describe('the draft auto-save', () => {
  it('is every 5 seconds', () => {
    expect(DRAFT_SAVE_INTERVAL_MS).toBe(5_000);
  });

  it('sends nothing for an untouched screen', async () => {
    render(<Harness stores={createBikriDraft('10-10-2026')} />);
    await vi.advanceTimersByTimeAsync(20_000);
    expect(draftsApi.draftsSave).not.toHaveBeenCalled();
    expect(draftsApi.draftsDiscard).not.toHaveBeenCalled();
  });

  it('sends the draft after 5 seconds, not before, and once for one change', async () => {
    const stores = createBikriDraft('10-10-2026');
    render(<Harness stores={stores} />);
    stores.header.getState().patch({ truckNo: 'LES-1234' });
    await vi.advanceTimersByTimeAsync(4_900);
    expect(draftsApi.draftsSave).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(200);
    expect(draftsApi.draftsSave).toHaveBeenCalledTimes(1);
    const [screen, payload] = vi.mocked(draftsApi.draftsSave).mock.calls[0] ?? [];
    expect(screen).toBe('bikri');
    expect(parseSnapshot(payload ?? '')?.header.truckNo).toBe('LES-1234');
    // Nothing changed since: the next ticks send nothing.
    await vi.advanceTimersByTimeAsync(15_000);
    expect(draftsApi.draftsSave).toHaveBeenCalledTimes(1);
    // A change in a lot goes in the next tick.
    const lot = stores.lots.getState().rows[0];
    if (lot === undefined) throw new Error('no lot');
    stores.lots.getState().updateTyped(lot.rowId, LOT.qty, '50');
    await vi.advanceTimersByTimeAsync(5_000);
    expect(draftsApi.draftsSave).toHaveBeenCalledTimes(2);
  });

  it('sends nothing when only the cursor moved: the text is the same as the last one sent', async () => {
    const stores = createBikriDraft('10-10-2026');
    render(<Harness stores={stores} />);
    stores.header.getState().patch({ truckNo: 'LES-1234' });
    await vi.advanceTimersByTimeAsync(5_000);
    expect(draftsApi.draftsSave).toHaveBeenCalledTimes(1);
    // A cursor move is a change of the lots store (its focus), not of what was typed.
    const lot = stores.lots.getState().rows[0];
    if (lot === undefined) throw new Error('no lot');
    stores.lots.getState().setFocus({ rowId: lot.rowId, field: LOT.seller });
    await vi.advanceTimersByTimeAsync(10_000);
    expect(draftsApi.draftsSave).toHaveBeenCalledTimes(1);
  });

  it('throws the draft away when the user empties the screen by hand', async () => {
    const stores = createBikriDraft('10-10-2026');
    render(<Harness stores={stores} />);
    stores.header.getState().patch({ truckNo: 'LES-1234' });
    await vi.advanceTimersByTimeAsync(5_000);
    expect(draftsApi.draftsSave).toHaveBeenCalledTimes(1);
    stores.header.getState().patch({ truckNo: '' });
    await vi.advanceTimersByTimeAsync(5_000);
    expect(draftsApi.draftsDiscard).toHaveBeenCalledWith('bikri');
    expect(draftsApi.draftsSave).toHaveBeenCalledTimes(1);
  });

  it('is cancelled by a save: nothing more is sent', async () => {
    const stores = createBikriDraft('10-10-2026');
    render(<Harness stores={stores} />);
    stores.header.getState().patch({ truckNo: 'LES-1234' });
    handle.stop();
    await vi.advanceTimersByTimeAsync(20_000);
    expect(draftsApi.draftsSave).not.toHaveBeenCalled();
  });

  it('throws away a draft that was on its way when the بکری was saved', async () => {
    const stores = createBikriDraft('10-10-2026');
    render(<Harness stores={stores} />);
    let arrive: () => void = () => undefined;
    vi.mocked(draftsApi.draftsSave).mockImplementation(
      () =>
        new Promise((resolve) => {
          arrive = () => {
            resolve(SAVED);
          };
        }),
    );
    stores.header.getState().patch({ truckNo: 'LES-1234' });
    await vi.advanceTimersByTimeAsync(5_000);
    expect(draftsApi.draftsSave).toHaveBeenCalledTimes(1);
    // The save of the بکری happens now, while the draft is still travelling.
    handle.stop();
    arrive();
    await vi.advanceTimersByTimeAsync(0);
    expect(draftsApi.draftsDiscard).toHaveBeenCalledWith('bikri');
  });

  it('tries again at the next tick when keeping the draft failed', async () => {
    const stores = createBikriDraft('10-10-2026');
    render(<Harness stores={stores} />);
    vi.mocked(draftsApi.draftsSave).mockRejectedValueOnce(new Error('busy'));
    stores.header.getState().patch({ truckNo: 'LES-1234' });
    await vi.advanceTimersByTimeAsync(5_000);
    await vi.advanceTimersByTimeAsync(5_000);
    expect(draftsApi.draftsSave).toHaveBeenCalledTimes(2);
  });
});
