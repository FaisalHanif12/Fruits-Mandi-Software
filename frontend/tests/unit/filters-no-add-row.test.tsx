// Plan 41, R5: a filter box has no "add" row. The report and list filters only find; a record is
// added where it is needed in an entry, never where a list is only narrowed. Checked on the filters
// of آج کی بکری (the seller and the item, behind "More filters"), in both languages, against the
// fake backend of Event 1, with the app's forms present (so the row would show if a filter had it).

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeBikri } from '../helpers/bikri';
import { named } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners } from '../helpers/partners';
import { message } from '../helpers/render';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/pukhta_khata/api');
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

describe.each(LANGUAGES)('the filter boxes (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
  });

  it('the seller and item filters of آج کی بکری find, and have no add row, however nothing matches', async () => {
    await renderApp('/bikri/today');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'layout-module-today')),
    });
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole('button', { name: named(message(language, 'common-filters-more')) }),
    );
    for (const id of ['glossary-beopari', 'glossary-qism-maal']) {
      const filter = await screen.findByRole('combobox', { name: named(message(language, id)) });
      await user.click(filter);
      await user.keyboard('zzz');
      expect(await screen.findByText(message(language, 'search-no-match'))).toBeInTheDocument();
      // No row: no id of an add row on the page, and F4 asks for nothing.
      expect(document.querySelector('[id$="-add"]')).toBeNull();
      await user.keyboard('{ArrowDown}{Enter}');
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
      await user.keyboard('{Control>}a{/Control}{Backspace}{Escape}');
      await waitFor(() => {
        expect(filter).toHaveAttribute('aria-expanded', 'false');
      });
    }
  });
});
