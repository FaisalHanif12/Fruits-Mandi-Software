// Closing a garden while its owner still owes (PRD FR-12.51, plan 80 Q-R4-2), in the Urdu and the English app.
// The installments paid are above the rent, so the owner owes money back, and a closed garden takes no refund.
// The first close (after the usual "close this garden?" box) is refused with ERR_OWNER_OWES_NOT_CONFIRMED and the
// value `owes_paisa`; the screen then asks a second question that shows the amount, with Back focused, and the
// yes sends the closing again with `confirm_owner_owes` true. No is nothing closed. Without a debt the close is
// one question, as before.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as gardensApi from '../../src/features/gardens/api';
import { LANGUAGES } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import * as urduApi from '../../src/shared/urdu/api';
import { named } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeGardens } from '../helpers/gardens';
import { EMPTY_PAIRS } from '../helpers/paged';
import { fakePartners, THREE_PARTNERS } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { refusal } from '../helpers/ui-error';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/gardens/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners(THREE_PARTNERS);
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('closing a garden whose owner still owes (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const said = (text: string) => (content: string) => visible(content) === text;

  /** The garden of E2E-40, whose close is refused until the owner-owes box was answered yes. */
  const openOwing = async () => {
    const fake = fakeGardens();
    const closed = vi.mocked(gardensApi.gardensClose).getMockImplementation();
    if (closed === undefined) throw new Error('the double has no close');
    vi.mocked(gardensApi.gardensClose).mockImplementation((input, confirmOwnerOwes) =>
      confirmOwnerOwes === true
        ? closed(input, confirmOwnerOwes)
        : Promise.reject(
            new CommandError(
              refusal([
                {
                  code: 'ERR_OWNER_OWES_NOT_CONFIRMED',
                  field: 'id',
                  values: { owes_paisa: 500_000 },
                },
              ]),
            ),
          ),
    );
    const view = await renderApp('/gardens/1');
    await screen.findByText(said(m('gardens-detail-title', { code: '0080' })));
    await screen.findByText(said(m('gardens-close-balance')));
    const closeButton = screen.getByRole('button', { name: m('gardens-close-button') });
    await waitFor(() => {
      expect(closeButton).toBeEnabled();
    });
    return { fake, closeButton, user: userEvent.setup(), ...view };
  };

  const firstBoxYes = () => screen.findByRole('button', { name: m('gardens-close-confirm') });

  it('asks a second question with the amount, with Back focused; the yes sends the closing again confirmed', async () => {
    const { fake, closeButton, user } = await openOwing();
    await user.click(closeButton);
    await user.click(await firstBoxYes());
    const second = await screen.findByRole('alertdialog');
    await within(second).findByText(said(m('gardens-close-owes-title')));
    // The amount the backend named, in whole rupees, and nothing closed yet.
    expect(visible(second.textContent)).toContain('5,000');
    expect(fake.saved.closings).toHaveLength(0);
    // Back has the focus, so a stray Enter never closes a garden that is owed money.
    expect(
      within(second).getByRole('button', {
        name: (name) => visible(name).startsWith(m('common-back')),
      }),
    ).toHaveFocus();
    await user.click(within(second).getByRole('button', { name: m('gardens-close-owes-confirm') }));
    await waitFor(() => {
      expect(fake.saved.closings).toHaveLength(1);
    });
    // Two calls: the first without the answer, the second with it.
    expect(vi.mocked(gardensApi.gardensClose).mock.calls.map(([, confirm]) => confirm)).toEqual([
      false,
      true,
    ]);
    expect(
      await screen.findByText(said(m('gardens-closed-toast', { code: '0080' }))),
    ).toBeInTheDocument();
  });

  it('No closes nothing and leaves no error behind', async () => {
    const { fake, closeButton, user } = await openOwing();
    await user.click(closeButton);
    await user.click(await firstBoxYes());
    const second = await screen.findByRole('alertdialog');
    await within(second).findByText(said(m('gardens-close-owes-title')));
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).toBeNull();
    });
    expect(fake.saved.closings).toHaveLength(0);
    expect(vi.mocked(gardensApi.gardensClose)).toHaveBeenCalledTimes(1);
    expect(screen.queryByRole('alert')).toBeNull();
    // The day typed is still there, ready to ask again.
    expect(screen.getByRole('textbox', { name: named(m('gardens-close-date')) })).toHaveValue(
      '10-10-2026',
    );
  });

  it('the refusal has words in the app language, for a screen that shows it as an error', () => {
    expect(m('err-owner-owes-not-confirmed', { owes: '5,000' })).not.toBe('');
  });
});
