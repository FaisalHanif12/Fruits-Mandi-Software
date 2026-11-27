// The one box of every cancel (plan 100 C, PRD FR-7.93), in the Urdu and the English app: it asks the
// reason, the Owner's sign and the 4-digit partner code, all required; only digits go into the code;
// nothing is sent while one is missing; the backend's refusal of a wrong code or sign shows under its
// field; and when no code was made yet it offers the way to make one. The secrets go only to onConfirm.

import { Localized } from '@fluent/react';
import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import type { UiError } from '../../src/bindings/UiError';
import { LANGUAGES, type Language } from '../../src/i18n';
import { ConfirmCancelDialog } from '../../src/shared/ui';
import { message, renderIn, visible } from '../helpers/render';
import { refusal } from '../helpers/ui-error';

function box(language: Language, error: UiError | null = null) {
  const onConfirm = vi.fn();
  renderIn(
    language,
    <ConfirmCancelDialog
      open
      onOpenChange={() => undefined}
      title={<Localized id="vouchers-cancel-title" />}
      description={<Localized id="cancel-reason-label" />}
      confirmLabel={<Localized id="vouchers-cancel-confirm" />}
      error={error}
      busy={false}
      onConfirm={onConfirm}
    />,
  );
  return { onConfirm };
}

describe.each(LANGUAGES)('the cancel box (%s)', (language) => {
  const m = (id: string) => message(language, id);
  const field = (id: string) =>
    screen.getByLabelText((label) => visible(label) === m(id), { selector: 'input' });
  const dialog = () => screen.getByRole('alertdialog');
  const press = (user: ReturnType<typeof userEvent.setup>) =>
    user.click(
      within(dialog()).getByRole('button', {
        name: (name) => visible(name) === m('vouchers-cancel-confirm'),
      }),
    );

  it('sends nothing until the reason, the sign and the four digits are all there', async () => {
    const user = userEvent.setup();
    const { onConfirm } = box(language);
    await press(user);
    expect(onConfirm).not.toHaveBeenCalled();
    expect(visible(dialog().textContent)).toContain(m('err-required'));
    expect(visible(dialog().textContent)).toContain(m('err-partner-code-invalid'));

    // Urdu letters pass through the Urdu field unchanged.
    const reason = language === 'ur' ? 'دو بار' : 'typed twice';
    await user.type(field('cancel-reason-label'), reason);
    await user.type(field('cancel-sign-label'), 'adnan');
    await user.type(field('cancel-code-label'), '48');
    await press(user);
    expect(onConfirm).not.toHaveBeenCalled();

    // Letters never go into the code; it stops at four digits. (Coming back to a field selects its
    // whole value, FR-7.15, so the code is typed again from the start.)
    await user.type(field('cancel-code-label'), 'x48 26 9');
    expect(field('cancel-code-label')).toHaveValue('4826');
    await press(user);
    expect(onConfirm).toHaveBeenCalledTimes(1);
    expect(onConfirm).toHaveBeenCalledWith({ reason, sign: 'adnan', partner_code: '4826' });
  });

  it('the code is hidden as it is typed', () => {
    box(language);
    expect(field('cancel-code-label')).toHaveAttribute('type', 'password');
  });

  it.each([
    ['ERR_PARTNER_CODE_WRONG', 'partner_code', 'cancel-code-label'],
    ['ERR_SIGN_WRONG', 'sign', 'cancel-sign-label'],
  ])('the backend’s %s shows under its field', (code, path, labelId) => {
    box(language, refusal([{ code, field: path, values: { tries_left: '9' } }]));
    const input = field(labelId);
    expect(input).toHaveAttribute('aria-invalid', 'true');
    const described = (input.getAttribute('aria-describedby') ?? '')
      .split(' ')
      .map((id) => document.getElementById(id)?.textContent ?? '')
      .join(' ');
    expect(described).not.toContain('ERR_');
    expect(visible(described).trim().length).toBeGreaterThan(3);
  });

  it('with no partner code made yet it says so and offers Settings', () => {
    box(language, refusal([{ code: 'ERR_PARTNER_CODE_NOT_SET', field: 'partner_code' }]));
    expect(visible(dialog().textContent)).toContain(m('cancel-no-code'));
    expect(
      within(dialog()).getByRole('button', {
        name: (name) => visible(name) === m('cancel-make-code'),
      }),
    ).toBeInTheDocument();
  });
});
