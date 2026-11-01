// The partner code (plan 44, PRD FR-17.11, E2E-46), in the Urdu and the English app, against the
// backend's own files: the section opens only with a 4-digit code; the first time the code is made
// with the Owner's password, with no way round; a wrong code says how many tries are left; after five
// there is a wait; leaving the section closes it; and wherever partner figures are left out a box asks
// for the code instead of showing a 0.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as partnersApi from '../../src/features/partners/api';
import { LANGUAGES } from '../../src/i18n';
import { PartnerStrip } from '../../src/features/dashboard/components/PartnerStrip';
import { ErrorState } from '../../src/shared/ui';
import { CommandError } from '../../src/shared/api';
import { named } from '../helpers/cashbook-screen';
import { leaveAppWindow, renderApp } from '../helpers/app';
import {
  PARTNER_CODE_SET,
  PARTNER_CODE_STATUS_LOCKED,
  PARTNER_CODE_STATUS_NOT_SET,
  PARTNER_CODE_STATUS_UNLOCKED,
  PARTNER_CODE_STATUS_WAITING,
} from '../helpers/edit-real';
import { preparePartnerMoney } from '../helpers/partner-money';
import { message, renderIn, visible } from '../helpers/render';
import { refusal } from '../helpers/ui-error';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/cashbook/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/pukhta_khata/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  window.localStorage.clear();
  vi.clearAllMocks();
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the partner code (%s)', (language) => {
  beforeEach(() => {
    preparePartnerMoney(language);
  });

  async function openPartners() {
    const app = await renderApp('/partners');
    return { ...app, user: userEvent.setup() };
  }

  const field = (label: string) =>
    screen.findByLabelText((name) => visible(name).startsWith(message(language, label)));

  it('no code yet: the section asks to make one, and shows nothing else (no way round it)', async () => {
    vi.mocked(partnersApi.partnersCodeStatus).mockResolvedValue(PARTNER_CODE_STATUS_NOT_SET);
    await openPartners();
    expect(
      await screen.findByRole('heading', {
        name: named(message(language, 'partners-code-set-title')),
      }),
    ).toBeInTheDocument();
    // The overview did not load behind the card.
    expect(partnersApi.partnersOverview).not.toHaveBeenCalled();
    expect(
      screen.queryByRole('heading', { level: 1, name: named(message(language, 'partners-title')) }),
    ).toBeNull();
  });

  it('the code is made with the Owner’s password and the code twice; the section then opens', async () => {
    vi.mocked(partnersApi.partnersCodeStatus).mockResolvedValue(PARTNER_CODE_STATUS_NOT_SET);
    vi.mocked(partnersApi.partnersCodeSet).mockImplementation(() => {
      vi.mocked(partnersApi.partnersCodeStatus).mockResolvedValue(PARTNER_CODE_STATUS_UNLOCKED);
      return Promise.resolve(PARTNER_CODE_SET);
    });
    const { user } = await openPartners();
    await user.type(await field('partners-code-password'), 'secret1');
    await user.type(await field('partners-code-new'), '4826');
    await user.type(await field('partners-code-again'), '4826');
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'partners-code-set-button')) }),
    );
    await waitFor(() => {
      expect(partnersApi.partnersCodeSet).toHaveBeenCalledWith({
        password: 'secret1',
        code: '4826',
        code_again: '4826',
      });
    });
    expect(
      await screen.findByRole('heading', {
        level: 1,
        name: named(message(language, 'partners-title')),
      }),
    ).toBeInTheDocument();
  });

  it('two codes that differ are told under the second one, and nothing is made', async () => {
    vi.mocked(partnersApi.partnersCodeStatus).mockResolvedValue(PARTNER_CODE_STATUS_NOT_SET);
    vi.mocked(partnersApi.partnersCodeSet).mockRejectedValue(
      new CommandError(
        refusal([{ code: 'ERR_PARTNER_CODES_DIFFER', field: 'code_again', values: {} }]),
      ),
    );
    const { user } = await openPartners();
    await user.type(await field('partners-code-password'), 'secret1');
    await user.type(await field('partners-code-new'), '4826');
    await user.type(await field('partners-code-again'), '4827');
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'partners-code-set-button')) }),
    );
    expect(
      await screen.findByText(visibleMatch(message(language, 'err-partner-codes-differ'))),
    ).toBeInTheDocument();
  });

  it('the code is only digits, at most four', async () => {
    vi.mocked(partnersApi.partnersCodeStatus).mockResolvedValue(PARTNER_CODE_STATUS_LOCKED);
    const { user } = await openPartners();
    const box = await field('partners-code-field');
    await user.type(box, '48a2x6789');
    expect(box).toHaveValue('4826');
  });

  it('a set code asks for it every time: the right code opens the overview (E2E-46)', async () => {
    vi.mocked(partnersApi.partnersCodeStatus).mockResolvedValue(PARTNER_CODE_STATUS_LOCKED);
    vi.mocked(partnersApi.partnersCodeUnlock).mockImplementation(() => {
      vi.mocked(partnersApi.partnersCodeStatus).mockResolvedValue(PARTNER_CODE_STATUS_UNLOCKED);
      return Promise.resolve(PARTNER_CODE_STATUS_UNLOCKED);
    });
    const { user } = await openPartners();
    await user.type(await field('partners-code-field'), '4826');
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'partners-code-open')) }),
    );
    await waitFor(() => {
      expect(partnersApi.partnersCodeUnlock).toHaveBeenCalledWith('4826');
    });
    expect(
      await screen.findByRole('heading', {
        level: 1,
        name: named(message(language, 'partners-title')),
      }),
    ).toBeInTheDocument();
  });

  it('a wrong code says how many tries are left and clears the box', async () => {
    vi.mocked(partnersApi.partnersCodeStatus).mockResolvedValue(PARTNER_CODE_STATUS_LOCKED);
    vi.mocked(partnersApi.partnersCodeUnlock).mockRejectedValue(
      new CommandError(
        refusal([{ code: 'ERR_PARTNER_CODE_WRONG', field: 'code', values: { tries_left: 4 } }]),
      ),
    );
    const { user } = await openPartners();
    const box = await field('partners-code-field');
    await user.type(box, '1111');
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'partners-code-open')) }),
    );
    expect(
      await screen.findByText(
        visibleMatch(message(language, 'err-partner-code-wrong', { tries_left: 4 })),
      ),
    ).toBeInTheDocument();
    expect(box).toHaveValue('');
  });

  it('after the wrong tries there is a wait: the button is off and the seconds are told', async () => {
    vi.mocked(partnersApi.partnersCodeStatus).mockResolvedValue(PARTNER_CODE_STATUS_WAITING);
    const { user } = await openPartners();
    await user.type(await field('partners-code-field'), '4826');
    expect(
      screen.getByRole('button', { name: named(message(language, 'partners-code-open')) }),
    ).toBeDisabled();
    expect(visible(document.body.textContent)).toContain(
      message(language, 'partners-code-wait', { seconds: 60 }),
    );
  });

  it('a forgotten code: the reset card asks for the password or the recovery code and a new code', async () => {
    vi.mocked(partnersApi.partnersCodeStatus).mockResolvedValue(PARTNER_CODE_STATUS_LOCKED);
    vi.mocked(partnersApi.partnersCodeReset).mockResolvedValue(PARTNER_CODE_SET);
    const { user } = await openPartners();
    await user.click(
      await screen.findByRole('button', { name: named(message(language, 'partners-code-forgot')) }),
    );
    await user.type(await field('partners-code-reset-secret'), 'RC-1234');
    await user.type(await field('partners-code-new'), '9753');
    await user.type(await field('partners-code-again'), '9753');
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'partners-code-reset-button')) }),
    );
    await waitFor(() => {
      expect(partnersApi.partnersCodeReset).toHaveBeenCalledWith({
        password_or_recovery: 'RC-1234',
        code: '9753',
        code_again: '9753',
      });
    });
  });

  it('leaving the section closes it again (the route-change lock)', async () => {
    vi.mocked(partnersApi.partnersCodeStatus).mockResolvedValue(PARTNER_CODE_STATUS_UNLOCKED);
    vi.mocked(partnersApi.partnersCodeLock).mockResolvedValue(PARTNER_CODE_STATUS_LOCKED);
    const { router } = await openPartners();
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'partners-title')),
    });
    // Inside the section a move keeps it open.
    await router.navigate({ to: '/partners/capital' });
    expect(partnersApi.partnersCodeLock).not.toHaveBeenCalled();
    // Out of the section it is closed.
    await router.navigate({ to: '/bikri/book', search: {} });
    await waitFor(() => {
      expect(partnersApi.partnersCodeLock).toHaveBeenCalledTimes(1);
    });
  });
});

describe.each(LANGUAGES)('where partner figures are left out (%s)', (language) => {
  it('a locked strip on the dashboard is a box that asks for the code, never zero cards', () => {
    renderIn(language, <PartnerStrip partners={[]} locked />);
    expect(visible(document.body.textContent)).toContain(
      message(language, 'partners-code-box-text'),
    );
    expect(
      screen.getByRole('button', { name: named(message(language, 'partners-code-enter')) }),
    ).toBeInTheDocument();
  });

  it('a screen the code keeps closed asks for the code instead of showing a failure', () => {
    renderIn(
      language,
      <ErrorState
        title={message(language, 'common-retry')}
        error={refusal([{ code: 'ERR_PARTNER_LOCKED', field: null, values: {} }])}
        onRetry={() => undefined}
      />,
    );
    expect(visible(document.body.textContent)).toContain(
      message(language, 'partners-code-box-text'),
    );
    expect(visible(document.body.textContent)).not.toContain(
      message(language, 'err-partner-locked'),
    );
  });

  it('every message of the partner code errors has words in this language', () => {
    for (const id of [
      'err-partner-locked',
      'err-partner-code-not-set',
      'err-partner-code-already-set',
      'err-partner-code-wrong',
      'err-partner-code-invalid',
      'err-partner-codes-differ',
    ]) {
      expect(() => message(language, id, { tries_left: 1 })).not.toThrow();
    }
  });
});

/** A matcher for the element whose own text is the message. */
function visibleMatch(text: string) {
  return (content: string) => visible(content) === text;
}
