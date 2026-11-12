// Test helper: the box of "reopen the day and fix" (PRD FR-21.4) answered, and the date the screen asked the
// backend to reopen. `day_fix_open` reads the date as typed, DD-MM-YYYY, and refuses the other with
// ERR_DATE_INVALID (Umar, 13-11-2026), so every screen that starts the flow has a test that reads this date.
// The test file mocks `shared/edit/api`; this reads its `dayFixOpen`.

import { screen, waitFor, within } from '@testing-library/react';
import type userEvent from '@testing-library/user-event';
import { expect, vi } from 'vitest';
import type { Language } from '../../src/i18n';
import * as editApi from '../../src/shared/edit/api';
import { named } from './bikri-screen';
import { DAY_FIX_OPEN_E2E_45 } from './edit-real';
import { message } from './render';

/** A date as the backend reads a typed one: DD-MM-YYYY. */
export const TYPED_DATE = /^\d{2}-\d{2}-\d{4}$/;

/** The backend opens the days; call this before the box is confirmed. */
export function backendReopens(): void {
  vi.mocked(editApi.dayFixOpen).mockResolvedValue(DAY_FIX_OPEN_E2E_45);
}

/**
 * Types the reason in the open box, confirms it, and answers the date the screen asked to reopen.
 * The date must be DD-MM-YYYY; `expected` also pins which day when the test knows it.
 */
export async function confirmFixDay(
  language: Language,
  user: ReturnType<typeof userEvent.setup>,
  expected?: string,
): Promise<string> {
  const dialog = await screen.findByRole('alertdialog');
  await user.type(
    within(dialog).getByRole('combobox', { name: named(message(language, 'edit-fix-label')) }),
    'wrong amount{Enter}',
  );
  await waitFor(() => {
    expect(editApi.dayFixOpen).toHaveBeenCalledTimes(1);
  });
  const call = vi.mocked(editApi.dayFixOpen).mock.calls[0];
  const date = call?.[0] ?? '';
  expect(date).toMatch(TYPED_DATE);
  if (expected !== undefined) expect(date).toBe(expected);
  expect(call?.[1]).toBe('wrong amount');
  return date;
}
