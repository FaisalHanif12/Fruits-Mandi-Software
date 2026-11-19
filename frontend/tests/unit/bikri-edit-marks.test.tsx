// The marks of earlier edits on the cells of a sale book open for edit (plan 44, PRD FR-7.71), in the
// Urdu and the English app: the history of the entry (the backend's own file of E2E-13: اکرم's rate
// 600 to 610) puts a corner mark on the rate cell of his line and on no other cell; an entry never
// edited asks for no history.

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import * as editApi from '../../src/shared/edit/api';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeBikri } from '../helpers/bikri';
import { fakePlan16 } from '../helpers/bikri-plan16';
import { cellOf, named, salesGrid, textOf } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { AUDIT_EDIT_HISTORY_BIKRI } from '../helpers/edit-real';
import { EMPTY_PAIRS } from '../helpers/paged';
import { fakePartners } from '../helpers/partners';
import { message } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/shared/edit/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the marks on the cells of a sale book open for edit (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
    fakePlan16();
    vi.mocked(editApi.auditEditStamp).mockResolvedValue([]);
  });

  const openEdit = async () => {
    const view = await renderApp('/bikri/book?id=41&edit=7');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'bikri-edit-title', { number: 1 })),
    });
    await waitFor(() => {
      expect(textOf(salesGrid(language), 0, 'buyer')).not.toBe('');
    });
    return view;
  };

  it('the rate of the line an edit changed has a mark; the cells next to it have none', async () => {
    vi.mocked(editApi.auditEditHistory).mockResolvedValue(AUDIT_EDIT_HISTORY_BIKRI);
    await openEdit();
    await waitFor(() => {
      expect(cellOf(salesGrid(language), 0, 'rate')).toHaveClass('cell-marked');
    });
    expect(cellOf(salesGrid(language), 0, 'qty')).not.toHaveClass('cell-marked');
    expect(cellOf(salesGrid(language), 0, 'buyer')).not.toHaveClass('cell-marked');
    expect(editApi.auditEditHistory).toHaveBeenCalledWith('bikri', expect.any(Number));
  });

  it('an entry whose history is empty marks nothing', async () => {
    vi.mocked(editApi.auditEditHistory).mockResolvedValue({
      entity: 'bikri',
      id: 41,
      versions: [],
    });
    await openEdit();
    await waitFor(() => {
      expect(editApi.auditEditHistory).toHaveBeenCalled();
    });
    expect(document.querySelectorAll('.cell-marked')).toHaveLength(0);
  });
});
