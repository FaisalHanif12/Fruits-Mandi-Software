// The box that asks before an entry takes cash in hand below zero (PRD FR-6.7). Focus starts on
// "No", so Enter out of habit never saves a mistake; Esc is No. Urdu and English.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import { CashConfirmDialog } from '../../src/shared/ui';
import { money } from '../helpers/accounts';
import { message, renderIn, visible } from '../helpers/render';

const named = (text: string) => (name: string) => visible(name) === text;

function Rig({
  onConfirm,
  startOpen = true,
  takeFocusBack = false,
}: {
  onConfirm: () => void;
  startOpen?: boolean;
  /** The screen puts focus on the amount field when the box closes (PRD FR-6.7). */
  takeFocusBack?: boolean;
}) {
  const [open, setOpen] = useState(startOpen);
  return (
    <>
      <input data-testid="amount" />
      <button
        type="button"
        data-testid="ask"
        onClick={() => {
          setOpen(true);
        }}
      />
      <CashConfirmDialog
        open={open}
        onOpenChange={setOpen}
        cashAfter={money(-5000)}
        onConfirm={onConfirm}
        onCloseFocus={
          takeFocusBack
            ? (event) => {
                event.preventDefault();
                document.querySelector<HTMLInputElement>('[data-testid="amount"]')?.focus();
              }
            : undefined
        }
      />
    </>
  );
}

describe.each(LANGUAGES)('the cash confirm box (%s)', (language) => {
  // The name of "No" ends with its key, Esc (Code Standards F29).
  const no = () =>
    screen.findByRole('button', {
      name: (name) => visible(name).startsWith(message(language, 'common-cash-confirm-no')),
    });
  const yes = () =>
    screen.findByRole('button', { name: named(message(language, 'common-cash-confirm-yes')) });

  it('says what cash will be, from the number it was given, and asks if the entry is correct', async () => {
    renderIn(language, <Rig onConfirm={vi.fn()} />);
    expect(
      await screen.findByText(named(message(language, 'common-cash-confirm-title'))),
    ).toBeInTheDocument();
    const text = visible(document.body.textContent);
    expect(text).toContain(message(language, 'common-cash-confirm-text', { amount: '-5,000' }));
    // The markup of the message is drawn, never shown as text.
    expect(text).not.toContain('<num>');
  });

  it('starts with focus on No, and Enter there does not save', async () => {
    const onConfirm = vi.fn();
    renderIn(language, <Rig onConfirm={onConfirm} />);
    expect(await no()).toHaveFocus();
    await userEvent.setup().keyboard('{Enter}');
    expect(onConfirm).not.toHaveBeenCalled();
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
    });
  });

  it('Esc is No: it closes the box and saves nothing', async () => {
    const onConfirm = vi.fn();
    renderIn(language, <Rig onConfirm={onConfirm} />);
    await no();
    await userEvent.setup().keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
    });
    expect(onConfirm).not.toHaveBeenCalled();
  });

  it('Yes saves, once, and closes the box', async () => {
    const onConfirm = vi.fn();
    renderIn(language, <Rig onConfirm={onConfirm} />);
    await userEvent.setup().click(await yes());
    expect(onConfirm).toHaveBeenCalledTimes(1);
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
    });
  });

  it('after No, focus goes back to the button that opened the box', async () => {
    const user = userEvent.setup();
    renderIn(language, <Rig onConfirm={vi.fn()} startOpen={false} />);
    const ask = screen.getByTestId('ask');
    await user.click(ask);
    await user.click(await no());
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
    });
    expect(ask).toHaveFocus();
  });

  it('the screen can take focus after No, for example to the amount field', async () => {
    const user = userEvent.setup();
    renderIn(language, <Rig onConfirm={vi.fn()} startOpen={false} takeFocusBack />);
    await user.click(screen.getByTestId('ask'));
    await user.click(await no());
    await waitFor(() => {
      expect(screen.getByTestId('amount')).toHaveFocus();
    });
  });
});
