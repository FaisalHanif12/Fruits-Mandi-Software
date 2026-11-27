// A record picked in a field is recorded as used again when an entry is saved (plan 100 I, PRD FR-7.41):
// once for each record, only after a save that went through, and the call that records a pick does not
// count as a save.

import { describe, expect, it, vi } from 'vitest';
import { createQueryClient } from '../../src/app/providers';
import { notePick } from '../../src/shared/search';
import { NOT_A_SAVE } from '../../src/shared/search/saved-picks';

function run(
  client: ReturnType<typeof createQueryClient>,
  works: boolean,
  meta?: typeof NOT_A_SAVE,
) {
  return client
    .getMutationCache()
    .build(client, {
      mutationFn: () => (works ? Promise.resolve(null) : Promise.reject(new Error('no'))),
      ...(meta === undefined ? {} : { meta }),
    })
    .execute(undefined)
    .catch(() => undefined);
}

describe('picked records are recorded again after a save', () => {
  it('once for each record, after the next save only', async () => {
    const client = createQueryClient();
    const akram = vi.fn(() => Promise.resolve(null));
    const banana = vi.fn(() => Promise.resolve(null));
    notePick('accounts|buyer|20', akram);
    notePick('accounts|buyer|20', akram);
    notePick('search|item|3', banana);
    // The call that records a pick, and a save that failed, record nothing.
    await run(client, true, NOT_A_SAVE);
    await run(client, false);
    expect(akram).not.toHaveBeenCalled();
    await run(client, true);
    expect(akram).toHaveBeenCalledTimes(1);
    expect(banana).toHaveBeenCalledTimes(1);
    await run(client, true);
    expect(akram).toHaveBeenCalledTimes(1);
  });
});
