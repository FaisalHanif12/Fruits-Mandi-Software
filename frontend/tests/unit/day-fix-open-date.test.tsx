// Reopen the day and fix (PRD FR-21.4): the date the backend is asked to reopen. Umar's bug of 13-11-2026:
// Edit on an entry of a closed day asked the reason, then said "یہ تاریخ درست نہیں" (ERR_DATE_INVALID on `date`
// of `day_fix_open`). The screens hold a day as the backend sent it, `YYYY-MM-DD`, but `day_fix_open` takes the
// date as typed, `DD-MM-YYYY` (the backend's own request file day_fix_open_e2e_45_request.json). The shared
// flow writes it as typed before it asks. These tests run the real flow and the real box in both languages,
// and the real wire function against the backend's request file.

import { Localized } from '@fluent/react';
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as editApi from '../../src/shared/edit/api';
import { useFixDayFlow } from '../../src/shared/edit';
import * as invoke from '../../src/shared/api';
import * as urduApi from '../../src/shared/urdu/api';
import { DAY_FIX_OPEN_E2E_45 } from '../helpers/edit-real';
import { named } from '../helpers/bikri-screen';
import { EMPTY_PAIRS } from '../helpers/paged';
import { message, renderIn } from '../helpers/render';
import request from '../fixtures/day_fix_open_e2e_45_request.json';

vi.mock('../../src/shared/edit/api');
vi.mock('../../src/shared/urdu/api');
vi.mock('../../src/shared/api', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/shared/api')>()),
  callCommand: vi.fn(),
}));

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(editApi.dayFixOpen).mockResolvedValue(DAY_FIX_OPEN_E2E_45);
  // The reason box is an Urdu text field (it suggests Urdu words as the user types).
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

/** A screen that does what every caller does: `start` with the day of the entry, and the box. */
function Caller({ date, then }: { date: string; then: () => void }) {
  const fix = useFixDayFlow();
  return (
    <>
      <Localized id="edit-action-fix-day">
        <button
          type="button"
          onClick={() => {
            fix.start({ date, ownerHref: '/bikri/book', then });
          }}
        />
      </Localized>
      {fix.dialog}
    </>
  );
}

async function confirmWithReason(language: Language, user: ReturnType<typeof userEvent.setup>) {
  await user.click(
    screen.getByRole('button', { name: named(message(language, 'edit-action-fix-day')) }),
  );
  const dialog = await screen.findByRole('alertdialog');
  await user.type(
    within(dialog).getByRole('combobox', { name: named(message(language, 'edit-fix-label')) }),
    'wrong rate{Enter}',
  );
  await user.keyboard('adnan{Enter}');
}

describe.each(LANGUAGES)('the date of the reopen and fix (%s)', (language) => {
  it('a day as the backend sent it (YYYY-MM-DD) goes to day_fix_open as typed (DD-MM-YYYY)', async () => {
    const then = vi.fn();
    renderIn(language, <Caller date="2026-10-10" then={then} />);
    await confirmWithReason(language, userEvent.setup());
    await waitFor(() => {
      expect(editApi.dayFixOpen).toHaveBeenCalledWith('10-10-2026', 'wrong rate', 'adnan');
    });
    // The days are open, so the screen goes on to open the entry for editing.
    await waitFor(() => {
      expect(then).toHaveBeenCalledTimes(1);
    });
  });

  it('a day already as typed (the opening date of an account form) goes as it is', async () => {
    renderIn(language, <Caller date="10-10-2026" then={vi.fn()} />);
    await confirmWithReason(language, userEvent.setup());
    await waitFor(() => {
      expect(editApi.dayFixOpen).toHaveBeenCalledWith('10-10-2026', 'wrong rate', 'adnan');
    });
  });

  it('the box says the day in the same words the backend will read', async () => {
    renderIn(language, <Caller date="2026-10-08" then={vi.fn()} />);
    await userEvent
      .setup()
      .click(screen.getByRole('button', { name: named(message(language, 'edit-action-fix-day')) }));
    const dialog = await screen.findByRole('alertdialog');
    expect(dialog.textContent).toContain('08-10-2026');
  });
});

describe('the wire of day_fix_open', () => {
  it('sends the backend’s own request file: date as typed, and the reason', async () => {
    const real = await vi.importActual<typeof import('../../src/shared/edit/api')>(
      '../../src/shared/edit/api',
    );
    vi.mocked(invoke.callCommand).mockResolvedValue(DAY_FIX_OPEN_E2E_45);
    await real.dayFixOpen(request.date, request.reason, request.sign);
    // The backend's request file: the date as typed, the reason and the Owner's sign (plan 73).
    expect(invoke.callCommand).toHaveBeenCalledWith('day_fix_open', { input: request });
  });
});
