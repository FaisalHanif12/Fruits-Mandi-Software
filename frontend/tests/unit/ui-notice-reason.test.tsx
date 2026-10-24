// Notice, ReasonDialog, SegmentedControl (disabled) and the "highlight first" choice of SuggestInput:
// the small shared pieces the voucher screen added (PRD FR-6.7, FR-7.68, FR-15.3). Urdu and English.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import { Notice, ReasonDialog, SegmentedControl } from '../../src/shared/ui';
import * as urduApi from '../../src/shared/urdu/api';
import { message, renderIn, visible } from '../helpers/render';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

const named = (text: string) => (name: string) => visible(name) === text;

// The words of the pieces under test come from Fluent messages that already exist, so no word is
// written in code (F31). Any message would do: only that the text arrives is checked.
const WARN = message('en', 'common-retry');
const FAILED = message('en', 'common-back');
const ACTION = message('en', 'common-more-columns');
const GROUP = message('en', 'vouchers-field-type');
const TITLE = message('en', 'vouchers-cancel-title');
const REASON = message('en', 'vouchers-cancel-reason');
const CONFIRM = message('en', 'vouchers-cancel-confirm');
const NOTE = message('en', 'common-cash-confirm-no');
const ONE = message('en', 'vouchers-type-receipt');
const TWO = message('en', 'vouchers-type-payment');

describe('Notice', () => {
  it('is a status for a warning and an alert for an error, with its action', async () => {
    const onClick = vi.fn();
    renderIn(
      'en',
      <>
        <Notice tone="warn" action={<button onClick={onClick}>{ACTION}</button>}>
          {WARN}
        </Notice>
        <Notice tone="error">{FAILED}</Notice>
      </>,
    );
    expect(screen.getByRole('status').textContent).toContain(WARN);
    expect(screen.getByRole('alert').textContent).toContain(FAILED);
    await userEvent.setup().click(screen.getByRole('button', { name: ACTION }));
    expect(onClick).toHaveBeenCalledTimes(1);
  });
});

describe('SegmentedControl', () => {
  function Rig({ disabled }: { disabled?: boolean }) {
    const [value, setValue] = useState<'a' | 'b'>('a');
    return (
      <SegmentedControl
        groupLabel={GROUP}
        value={value}
        onChange={setValue}
        disabled={disabled}
        options={[
          { value: 'a', label: ONE },
          { value: 'b', label: TWO },
        ]}
      />
    );
  }

  it('presses the chosen option', async () => {
    renderIn('en', <Rig />);
    const user = userEvent.setup();
    await user.click(screen.getByRole('button', { name: TWO }));
    expect(screen.getByRole('button', { name: TWO }).getAttribute('aria-pressed')).toBe('true');
    expect(screen.getByRole('button', { name: ONE }).getAttribute('aria-pressed')).toBe('false');
  });

  it('cannot be changed while disabled', async () => {
    renderIn('en', <Rig disabled />);
    const user = userEvent.setup();
    await user.click(screen.getByRole('button', { name: TWO }));
    expect(screen.getByRole('button', { name: ONE }).getAttribute('aria-pressed')).toBe('true');
    expect(screen.getByRole('button', { name: TWO }).hasAttribute('disabled')).toBe(true);
  });
});

describe.each(LANGUAGES)('ReasonDialog (%s)', (language) => {
  function Rig({ onConfirm, note }: { onConfirm: (reason: string) => void; note?: string }) {
    const [open, setOpen] = useState(true);
    return (
      <ReasonDialog
        open={open}
        onOpenChange={setOpen}
        title={TITLE}
        description={WARN}
        reasonLabel={REASON}
        confirmLabel={CONFIRM}
        error={null}
        busy={false}
        {...(note === undefined ? {} : { note })}
        onConfirm={onConfirm}
      />
    );
  }

  it('starts in the reason field and tells an empty reason without sending anything', async () => {
    const onConfirm = vi.fn();
    renderIn(language, <Rig onConfirm={onConfirm} />);
    const field = await screen.findByLabelText(REASON);
    await waitFor(() => {
      expect(document.activeElement).toBe(field);
    });
    const user = userEvent.setup();
    await user.keyboard('{Enter}');
    expect(onConfirm).not.toHaveBeenCalled();
    expect(await screen.findByText(named(message(language, 'err-required')))).toBeTruthy();
  });

  it('sends the trimmed reason on Enter and on the button', async () => {
    const onConfirm = vi.fn();
    renderIn(language, <Rig onConfirm={onConfirm} note={NOTE} />);
    const user = userEvent.setup();
    // Typed in English letters with a space after: the reason is kept exactly as typed.
    await user.type(await screen.findByLabelText(REASON), '  typed twice  ');
    await user.keyboard('{Enter}');
    expect(onConfirm).toHaveBeenLastCalledWith('typed twice');
    expect(screen.getByText(NOTE)).toBeTruthy();
    await user.click(screen.getByRole('button', { name: named(CONFIRM) }));
    expect(onConfirm).toHaveBeenCalledTimes(2);
  });

  it('closes on Esc and keeps nothing sent', async () => {
    const onConfirm = vi.fn();
    renderIn(language, <Rig onConfirm={onConfirm} />);
    await screen.findByLabelText(REASON);
    await userEvent.setup().keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByLabelText(REASON)).toBeNull();
    });
    expect(onConfirm).not.toHaveBeenCalled();
  });
});
