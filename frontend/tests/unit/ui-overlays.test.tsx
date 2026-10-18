// Select, Toast and ConfirmDialog in Urdu and English (plan 02 T3, design system 6,
// PRD FR-7.7 and FR-7.8). These render in a portal, so they set lang and dir themselves.

import { act, fireEvent, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import { directionOf } from '../../src/shared/i18n';
import { SHORTCUTS } from '../../src/shared/keyboard';
import {
  ConfirmDialog,
  Field,
  Select,
  TOAST_DURATION_MS,
  Toaster,
  showToast,
  useToastStore,
  type ConfirmTone,
} from '../../src/shared/ui';
import { message, renderIn } from '../helpers/render';

type Mode = 'cash' | 'bank';

afterEach(() => {
  useToastStore.setState({ toasts: [] });
  vi.useRealTimers();
});

function DialogHarness({
  language,
  tone,
  onConfirm,
}: {
  language: Language;
  tone: ConfirmTone;
  onConfirm: () => void;
}) {
  const [open, setOpen] = useState(false);
  return (
    <>
      <button
        type="button"
        data-testid="opener"
        onClick={() => {
          setOpen(true);
        }}
      />
      <ConfirmDialog
        open={open}
        onOpenChange={setOpen}
        tone={tone}
        title={message(language, 'common-retry')}
        description={message(language, 'err-internal')}
        confirmLabel={message(language, 'common-loading')}
        onConfirm={onConfirm}
      />
    </>
  );
}

describe.each(LANGUAGES)('overlays (%s)', (language) => {
  const direction = directionOf(language);

  it('Select shows the Fluent placeholder, opens a list in the page language, and reports the choice', async () => {
    const onChange = vi.fn();
    const options = [
      { value: 'cash' as Mode, label: message(language, 'common-back') },
      { value: 'bank' as Mode, label: message(language, 'common-retry') },
    ];
    renderIn(
      language,
      <Field label={message(language, 'common-loading')}>
        {(control) => (
          <Select {...control} value={undefined} onChange={onChange} options={options} />
        )}
      </Field>,
    );
    const trigger = screen.getByRole('combobox', { name: message(language, 'common-loading') });
    expect(trigger).toHaveTextContent(message(language, 'common-select-placeholder'));
    expect(trigger).toHaveClass('h-control', 'border-border-control');
    await userEvent.click(trigger);
    const list = await screen.findByRole('listbox');
    expect(list.closest('[lang]')).toHaveAttribute('lang', language);
    expect(list.closest('[dir]')).toHaveAttribute('dir', direction);
    await userEvent.click(screen.getByRole('option', { name: message(language, 'common-retry') }));
    expect(onChange).toHaveBeenCalledWith('bank');
  });

  it('Select read-only cannot open', () => {
    renderIn(
      language,
      <Select value="cash" onChange={vi.fn()} readOnly options={[{ value: 'cash', label: 'x' }]} />,
    );
    expect(screen.getByRole('combobox')).toBeDisabled();
    expect(screen.getByRole('combobox')).toHaveClass('bg-bg');
  });

  it('Toast shows at the end corner in the page language and goes by itself (FR-7.7)', () => {
    vi.useFakeTimers();
    renderIn(language, <Toaster />);
    act(() => {
      showToast('success', message(language, 'common-back'));
    });
    const toast = screen.getByText(message(language, 'common-back')).closest('li');
    expect(toast).toHaveAttribute('lang', language);
    expect(toast).toHaveAttribute('dir', direction);
    expect(toast).toHaveClass('bg-nav', 'text-on-primary');
    expect(toast?.querySelector('svg')).toHaveClass('text-money-in-on-nav');
    expect(toast?.closest('ol')).toHaveClass('fixed', 'end-6', 'bottom-7');
    expect(screen.getByRole('region')).toHaveAttribute(
      'aria-label',
      message(language, 'common-notifications'),
    );
    act(() => {
      vi.advanceTimersByTime(TOAST_DURATION_MS + 500);
    });
    expect(screen.queryByText(message(language, 'common-back'))).toBeNull();
    expect(useToastStore.getState().toasts).toHaveLength(0);
  });

  it('a toast with a button: the button and its key run the action once and close it (FR-7.47)', () => {
    const onAction = vi.fn();
    renderIn(language, <Toaster />);
    act(() => {
      showToast('success', message(language, 'common-back'), {
        durationMs: 10_000,
        action: {
          label: message(language, 'common-retry'),
          altText: message(language, 'common-retry'),
          shortcut: SHORTCUTS.undoSave,
          onAction,
        },
      });
    });
    const button = screen.getByRole('button', {
      name: (name) => name.startsWith(message(language, 'common-retry')),
    });
    expect(button).toHaveTextContent('Alt+Z');
    // The key works while the message is on screen, runs the action once and closes the message.
    act(() => {
      fireEvent.keyDown(document.body, { code: 'KeyZ', key: 'z', altKey: true });
    });
    expect(onAction).toHaveBeenCalledTimes(1);
    expect(screen.queryByText(message(language, 'common-back'))).toBeNull();
    // Gone with the message: the key does nothing any more.
    act(() => {
      fireEvent.keyDown(document.body, { code: 'KeyZ', key: 'z', altKey: true });
    });
    expect(onAction).toHaveBeenCalledTimes(1);
  });

  it('a toast with a button stays for the time it asks, not the usual 5 seconds', () => {
    vi.useFakeTimers();
    renderIn(language, <Toaster />);
    act(() => {
      showToast('success', message(language, 'common-back'), {
        durationMs: 10_000,
        action: {
          label: message(language, 'common-retry'),
          altText: message(language, 'common-retry'),
          onAction: vi.fn(),
        },
      });
    });
    act(() => {
      vi.advanceTimersByTime(TOAST_DURATION_MS + 500);
    });
    expect(screen.queryByText(message(language, 'common-back'))).not.toBeNull();
    act(() => {
      vi.advanceTimersByTime(TOAST_DURATION_MS + 500);
    });
    expect(screen.queryByText(message(language, 'common-back'))).toBeNull();
  });

  it('an error toast has the error icon and is announced at once', () => {
    renderIn(language, <Toaster />);
    act(() => {
      showToast('error', message(language, 'err-internal'));
    });
    const toast = screen.getByText(message(language, 'err-internal')).closest('li');
    expect(toast?.querySelector('svg')).toHaveClass('text-due-on-nav');
    expect(document.querySelector('[role="status"][aria-live="assertive"]')).not.toBeNull();
  });

  it('ConfirmDialog: says what will happen, the action has focus, it runs and closes', async () => {
    const onConfirm = vi.fn();
    renderIn(language, <DialogHarness language={language} tone="primary" onConfirm={onConfirm} />);
    await userEvent.click(screen.getByTestId('opener'));
    const dialog = screen.getByRole('alertdialog');
    expect(dialog).toHaveAttribute('lang', language);
    expect(dialog).toHaveAttribute('dir', direction);
    expect(dialog).toHaveTextContent(message(language, 'err-internal'));
    const action = screen.getByRole('button', { name: message(language, 'common-loading') });
    expect(action).toHaveFocus();
    expect(screen.getByRole('button', { name: /Esc/ })).toHaveTextContent(
      message(language, 'common-back'),
    );
    await userEvent.keyboard('{Enter}');
    expect(onConfirm).toHaveBeenCalledOnce();
    expect(screen.queryByRole('alertdialog')).toBeNull();
    expect(screen.getByTestId('opener')).toHaveFocus();
  });

  it('ConfirmDialog for a danger action: back has focus, Esc closes, nothing runs', async () => {
    const onConfirm = vi.fn();
    renderIn(language, <DialogHarness language={language} tone="danger" onConfirm={onConfirm} />);
    await userEvent.click(screen.getByTestId('opener'));
    const back = screen.getByRole('button', { name: new RegExp(message(language, 'common-back')) });
    expect(back).toHaveFocus();
    expect(screen.getByRole('button', { name: message(language, 'common-loading') })).toHaveClass(
      'text-due',
    );
    // Focus stays inside while the dialog is open.
    await userEvent.tab();
    await userEvent.tab();
    expect(screen.getByRole('alertdialog')).toContainElement(document.activeElement as HTMLElement);
    await userEvent.keyboard('{Escape}');
    expect(screen.queryByRole('alertdialog')).toBeNull();
    expect(onConfirm).not.toHaveBeenCalled();
    expect(screen.getByTestId('opener')).toHaveFocus();
  });
});
