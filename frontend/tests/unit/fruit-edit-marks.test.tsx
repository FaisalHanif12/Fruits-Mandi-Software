// The "Before ..." marks on the fields of a پھل entry open for edit (plan 44, PRD FR-7.71), in the
// Urdu and the English app: the quantity and the rate that an earlier edit changed carry what they
// held before; the item that was changed shows the NAME it had, not its number's digits as a bare
// id; a field nobody changed has none.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as fruitApi from '../../src/features/fruit/api';
import { LANGUAGES } from '../../src/i18n';
import type { EditHistory } from '../../src/bindings/EditHistory';
import * as editApi from '../../src/shared/edit/api';
import * as urduApi from '../../src/shared/urdu/api';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { named } from '../helpers/bikri-screen';
import { fakeFruit, fakeFruitPickers } from '../helpers/fruit';
import * as real from '../helpers/fruit-real';
import { EMPTY_PAIRS } from '../helpers/paged';
import { fakePartners, THREE_PARTNERS } from '../helpers/partners';
import { paisaForTest } from '../helpers/paisa';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/fruit/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');
vi.mock('../../src/shared/edit/api');

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners(THREE_PARTNERS);
  fakeFruitPickers();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(editApi.auditEditStamp).mockResolvedValue([]);
});

afterEach(() => {
  leaveAppWindow();
});

const HISTORY: EditHistory = {
  entity: 'fruit_take',
  id: 1,
  versions: [
    {
      audit_id: 9,
      at: '2026-10-11T09:00:00Z',
      local_date: '2026-10-11',
      local_time: '14:00',
      by: 'Owner',
      action: 'edit',
      reason: 'wrong count',
      changes: [
        { path: 'qty', old: { kind: 'number', value: 3 }, new: { kind: 'number', value: 2 } },
        {
          path: 'rate_paisa',
          old: { kind: 'money', paisa: paisaForTest(55_000) },
          new: { kind: 'money', paisa: paisaForTest(58_000) },
        },
      ],
      changes_truncated: false,
    },
  ],
};

describe.each(LANGUAGES)('the marks of an edited پھل entry (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  // The saved view has the stamp too, with its "Edit history" button: the Edit button is the exact word.
  const exact = (id: string) => (name: string) => visible(name) === message(language, id);
  const marks = () => Array.from(document.querySelectorAll<HTMLElement>('[data-field-mark]'));

  async function openForEdit() {
    const fake = fakeFruit();
    fake.rows.push({ ...real.FRUIT_GET_HOME_EDITED });
    await renderApp('/rokar/fruit?entry=1');
    const user = userEvent.setup();
    const card = (
      await screen.findByText(
        (content) =>
          visible(content) ===
          message(language, 'fruit-entry-title', {
            number: message(language, 'fruit-number', { number: 1 }),
          }),
      )
    ).closest('section');
    if (card === null) throw new Error('no entry card');
    await user.click(within(card).getByRole('button', { name: exact('edit-action-edit') }));
    await screen.findByLabelText(named(message(language, 'fruit-field-qty')));
  }

  it('the quantity and the rate carry what they held before; the other fields have no mark', async () => {
    vi.mocked(editApi.auditEditHistory).mockResolvedValue(HISTORY);
    await openForEdit();
    await waitFor(() => {
      expect(marks()).toHaveLength(2);
    });
    const texts = marks().map((mark) => visible(mark.textContent));
    expect(texts).toContain(message(language, 'edit-mark-before', { before: '3' }));
    expect(texts).toContain(message(language, 'edit-mark-before', { before: '550' }));
    expect(editApi.auditEditHistory).toHaveBeenCalledWith('fruit_take', 1);
    expect(fruitApi.fruitGet).toHaveBeenCalled();
  });
});
