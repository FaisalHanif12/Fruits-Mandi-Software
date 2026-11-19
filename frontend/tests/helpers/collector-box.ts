// A collector box lists the collectors only (Umar, plan 55 Section 11.1): the active staff whose job is
// collector, from a search scope of their own, never the whole staff with a filter on top. This opens a
// box and checks what it offers and what it asked for. The staff are the `STAFF_ROSTER` of the masters fake.

import { screen } from '@testing-library/react';
import type userEvent from '@testing-library/user-event';
import { expect, vi } from 'vitest';
import type { Language } from '../../src/i18n';
import * as searchApi from '../../src/shared/search/api';
import { visible } from './render';

type User = ReturnType<typeof userEvent.setup>;

const NAMES = {
  collector: { ur: 'اسلم', en: 'Aslam' },
  munshi: { ur: 'رفیق', en: 'Rafiq' },
  left: { ur: 'زاہد', en: 'Zahid' },
} as const;

export async function expectOnlyCollectors(
  user: User,
  box: HTMLElement,
  language: Language,
): Promise<void> {
  vi.mocked(searchApi.searchMasters).mockClear();
  vi.mocked(searchApi.searchRecent).mockClear();
  await user.click(box);
  const options = await screen.findAllByRole('option');
  const shown = options.map((option) => visible(option.textContent));
  // The collector is offered; the munshi and the collector who left are not.
  expect(shown.some((text) => text.includes(NAMES.collector[language]))).toBe(true);
  expect(shown.some((text) => text.includes(NAMES.munshi[language]))).toBe(false);
  expect(shown.some((text) => text.includes(NAMES.left[language]))).toBe(false);
  // The box asked the collectors' own list, and never the staff.
  const scopes = [
    ...vi.mocked(searchApi.searchMasters).mock.calls,
    ...vi.mocked(searchApi.searchRecent).mock.calls,
  ].map(([input]) => input.scope);
  expect(scopes.length).toBeGreaterThan(0);
  expect(new Set(scopes)).toEqual(new Set(['collector']));
}
