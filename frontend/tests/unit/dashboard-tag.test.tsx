// The tag (نشان, PRD 10.2, FR-7.40) on the dashboard: a seller in today's بکری and the most urgent
// record of the attention list are written with their tag after the name, in the language of the app
// and only through `displayName` (Code Standards F38). Against a fake backend; the two answers are
// the backend's own with a tag added to the one seller.

import { screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { AttentionList } from '../../src/bindings/AttentionList';
import type { Dashboard } from '../../src/bindings/Dashboard';
import { LANGUAGES, type Language } from '../../src/i18n';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { ATTENTION_18_10, DASHBOARD_WORKED_DAY, fakeDashboard } from '../helpers/dashboard';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/dashboard/api');

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
});

afterEach(() => {
  leaveAppWindow();
});

/** The worked day with the tag K on the seller of the first بکری (رحیم). */
const WITH_SELLER_TAG: Dashboard = {
  ...DASHBOARD_WORKED_DAY,
  bikri_today: DASHBOARD_WORKED_DAY.bikri_today.map((row, index) =>
    index === 0 ? { ...row, seller_tag: 'K' } : row,
  ),
};

/** The attention list with the tag BA on the most urgent record (کریم, the oldest payable). */
const WITH_FIRST_TAG: AttentionList = {
  ...ATTENTION_18_10,
  items: ATTENTION_18_10.items.map((item) =>
    item.first === null ? item : { ...item, first: { ...item.first, tag: 'BA' } },
  ),
};

async function openDashboard(language: Language) {
  resetClientState(language);
  signedIn(sessionUser({ language }));
  return renderApp('/');
}

describe.each(LANGUAGES)('the tag on the dashboard (%s)', (language) => {
  it('writes the seller of a بکری with his tag after his name', async () => {
    fakeDashboard({ dashboard: WITH_SELLER_TAG });
    await openDashboard(language);
    // A name is drawn in its own isolate (<bdi>), so the table cell around it is not matched twice.
    const written = (text: string) => (_: string, element: Element | null) =>
      element?.tagName === 'BDI' && visible(element.textContent) === text;
    const name = language === 'ur' ? 'رحیم' : 'Rahim';
    expect(await screen.findByText(written(`${name} K`))).toBeVisible();
    // The other seller has no tag: his name stands alone.
    const other = language === 'ur' ? 'کریم' : 'Karim';
    expect(screen.queryByText(written(`${other} K`))).toBeNull();
    expect(screen.getByText(written(other))).toBeVisible();
  });

  it('writes the most urgent record of the attention list with his tag', async () => {
    fakeDashboard({
      attention: WITH_FIRST_TAG,
      dashboard: { ...DASHBOARD_WORKED_DAY, attention: WITH_FIRST_TAG },
    });
    await openDashboard(language);
    const name = language === 'ur' ? 'کریم BA' : 'Karim BA';
    const wanted = message(language, 'attention-open-first', { name });
    const button = await screen.findByRole('button', {
      name: (label) => visible(label) === wanted,
    });
    expect(
      within(button).getByText((_, element) => visible(element?.textContent) === name),
    ).toBeVisible();
  });
});
