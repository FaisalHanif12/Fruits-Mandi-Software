// Field and the typed controls, in Urdu and English (plan 02 T3, design system 6,
// Code Standards 9.1 "New screen").

import { fireEvent, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import { useGroupingStore } from '../../src/shared/format';
import { CodeInput, DateInput, Field, MoneyInput, QtyInput, TextInput } from '../../src/shared/ui';
import { message, renderIn } from '../helpers/render';

type Control = typeof MoneyInput;

/** A field with its own text state, reporting every value it would send. */
function Harness({
  control: Input,
  initial = '',
  onValue,
}: {
  control: Control;
  initial?: string;
  onValue: (text: string) => void;
}) {
  const [value, setValue] = useState(initial);
  return (
    <Field label={<span data-testid="label" />} hint={<span data-testid="hint" />}>
      {(props) => (
        <Input
          {...props}
          value={value}
          onChange={(text) => {
            setValue(text);
            onValue(text);
          }}
        />
      )}
    </Field>
  );
}

afterEach(() => {
  useGroupingStore.setState({ grouping: 'international' });
});

describe.each(LANGUAGES)('Field (%s)', (language) => {
  it('puts the label above, linked to the control, and the hint below', () => {
    renderIn(
      language,
      <Field label={message(language, 'err-required')} hint={message(language, 'common-loading')}>
        {(control) => <TextInput {...control} value="" onChange={vi.fn()} />}
      </Field>,
    );
    const input = screen.getByLabelText(message(language, 'err-required'));
    const hint = screen.getByText(message(language, 'common-loading'));
    expect(input).toHaveAttribute('aria-describedby', hint.id);
    expect(input).not.toHaveAttribute('aria-invalid', 'true');
    expect(input).toHaveClass('h-control', 'rounded-1', 'border-border-control', 'bg-surface');
    // Label, control, then note: the order a reader meets them in both directions.
    const order = input.closest('div')?.children;
    expect(order?.[0]?.tagName).toBe('LABEL');
    expect(order?.[2]).toBe(hint);
  });

  it('shows an error in red with an icon and words, instead of the hint', () => {
    const words = message(language, 'err-decimals-not-allowed');
    renderIn(
      language,
      <Field
        label={message(language, 'common-back')}
        hint={message(language, 'common-retry')}
        error={words}
      >
        {(control) => <MoneyInput {...control} value="15000.50" onChange={vi.fn()} />}
      </Field>,
    );
    const input = screen.getByRole('textbox');
    const note = screen.getByText(words).closest('p');
    expect(input).toHaveAttribute('aria-invalid', 'true');
    expect(input).toHaveClass('border-due', 'bg-due-soft');
    expect(input).toHaveAttribute('aria-describedby', note?.id);
    expect(note).toHaveClass('text-due');
    expect(note?.querySelector('svg')).not.toBeNull();
    expect(screen.queryByText(message(language, 'common-retry'))).toBeNull();
  });

  it('shows a read-only value on the page ground', () => {
    renderIn(
      language,
      <Field label={message(language, 'common-back')} readOnly>
        {(control) => <MoneyInput {...control} value="12880" onChange={vi.fn()} />}
      </Field>,
    );
    const input = screen.getByRole('textbox');
    expect(input).toHaveAttribute('readonly');
    expect(input).toHaveClass('bg-bg');
    expect(input).toHaveValue('12,880');
  });
});

describe.each(LANGUAGES)('number fields (%s)', (language) => {
  // The backend takes out the LRM mark (U+200E) before it parses number text. That is safe only
  // while every number field is written left to right, in the Urdu app too (plan 02 Section 4.6).
  it.each([
    ['MoneyInput', MoneyInput],
    ['QtyInput', QtyInput],
    ['DateInput', DateInput],
    ['CodeInput', CodeInput],
  ] as const)('%s writes its input left to right inside the app direction', (_name, Input) => {
    renderIn(language, <Input value="12000" onChange={vi.fn()} />);
    expect(screen.getByTestId('language-root')).toHaveAttribute(
      'dir',
      language === 'ur' ? 'rtl' : 'ltr',
    );
    const input = screen.getByRole('textbox');
    expect(input).toHaveAttribute('dir', 'ltr');
    expect(input).toHaveAttribute('lang', 'en');
  });

  it('MoneyInput shows separators while typing and sends the digits as text (FR-7.20)', async () => {
    const onValue = vi.fn();
    renderIn(language, <Harness control={MoneyInput} onValue={onValue} />);
    const input = screen.getByRole('textbox');
    await userEvent.type(input, '1316040');
    expect(input).toHaveValue('1,316,040');
    expect(onValue).toHaveBeenLastCalledWith('1316040');
    // Latin digits, left to right, on the physical right edge in both languages (F17).
    expect(input).toHaveAttribute('dir', 'ltr');
    expect(input).toHaveAttribute('lang', 'en');
    expect(input).toHaveClass('type-number-field');
  });

  it('MoneyInput follows the lakh grouping setting', async () => {
    useGroupingStore.setState({ grouping: 'lakh' });
    const onValue = vi.fn();
    renderIn(language, <Harness control={MoneyInput} onValue={onValue} />);
    const input = screen.getByRole('textbox');
    await userEvent.type(input, '1316040');
    expect(input).toHaveValue('13,16,040');
    expect(onValue).toHaveBeenLastCalledWith('1316040');
  });

  it('keeps the cursor after the digit just typed when a separator appears', async () => {
    renderIn(language, <Harness control={MoneyInput} onValue={vi.fn()} />);
    const input = screen.getByRole<HTMLInputElement>('textbox');
    await userEvent.type(input, '1234');
    expect(input).toHaveValue('1,234');
    expect(input.selectionStart).toBe('1,234'.length);
  });

  it.each([
    ['QtyInput', QtyInput],
    ['MoneyInput', MoneyInput],
  ] as const)(
    '%s keeps a sum, Urdu digits and Arabic-Indic digits as typed',
    async (_name, Input) => {
      for (const typed of ['20+25', '15000.50', '۱۲۰۰', '١٢٠٠', '12a']) {
        const onValue = vi.fn();
        const view = renderIn(language, <Harness control={Input} onValue={onValue} />);
        const input = screen.getByRole('textbox');
        await userEvent.type(input, typed);
        expect(input).toHaveValue(typed);
        expect(onValue).toHaveBeenLastCalledWith(typed);
        view.unmount();
      }
    },
  );

  it('never puts a separator into the value, even when a decimal point follows a group', async () => {
    const onValue = vi.fn();
    renderIn(language, <Harness control={MoneyInput} onValue={onValue} />);
    const input = screen.getByRole('textbox');
    await userEvent.type(input, '15000');
    expect(input).toHaveValue('15,000');
    await userEvent.type(input, '.50');
    expect(input).toHaveValue('15000.50');
    expect(onValue).toHaveBeenLastCalledWith('15000.50');
  });

  // Plan 02 4.7, finding R02-6. Commas the field adds for display are never sent. A comma the
  // user types or pastes is sent exactly as typed, and the backend checks the grouping.
  it('typing 12000 shows 12,000 and sends 12000', async () => {
    const onValue = vi.fn<(text: string) => void>();
    renderIn(language, <Harness control={MoneyInput} onValue={onValue} />);
    const input = screen.getByRole('textbox');
    await userEvent.type(input, '12000');
    expect(input).toHaveValue('12,000');
    expect(onValue).toHaveBeenLastCalledWith('12000');
    expect(onValue.mock.calls.map((call) => call[0])).toEqual(['1', '12', '120', '1200', '12000']);
  });

  it.each([
    ['MoneyInput', MoneyInput],
    ['QtyInput', QtyInput],
  ] as const)('%s sends a pasted 1,200 as 1,200 and shows it as pasted', (_name, Input) => {
    const onValue = vi.fn();
    renderIn(language, <Harness control={Input} onValue={onValue} />);
    fireEvent.change(screen.getByRole('textbox'), { target: { value: '1,200' } });
    expect(onValue).toHaveBeenLastCalledWith('1,200');
    expect(screen.getByRole('textbox')).toHaveValue('1,200');
  });

  it('typing 1,200 key by key sends 1,200 at the end', async () => {
    const onValue = vi.fn();
    renderIn(language, <Harness control={MoneyInput} onValue={onValue} />);
    const input = screen.getByRole('textbox');
    await userEvent.type(input, '1,200');
    expect(input).toHaveValue('1,200');
    expect(onValue).toHaveBeenLastCalledWith('1,200');
  });

  it.each([
    ['MoneyInput', '850,50', MoneyInput],
    ['MoneyInput', '1,00,0', MoneyInput],
    ['QtyInput', '1,2', QtyInput],
  ] as const)(
    '%s sends %s exactly as typed and pasted, for the backend to refuse (R02-6)',
    async (_name, typed, Input) => {
      for (const how of ['type', 'paste'] as const) {
        const onValue = vi.fn();
        const view = renderIn(language, <Harness control={Input} onValue={onValue} />);
        const input = screen.getByRole('textbox');
        if (how === 'type') await userEvent.type(input, typed);
        else fireEvent.change(input, { target: { value: typed } });
        // Not 85050, not 1000, not 12: the commas are the user's own and stay.
        expect(onValue).toHaveBeenLastCalledWith(typed);
        expect(input).toHaveValue(typed);
        view.unmount();
      }
    },
  );

  it('does not hide a typed comma by regrouping the digits around it', async () => {
    const onValue = vi.fn();
    renderIn(language, <Harness control={MoneyInput} initial="85050" onValue={onValue} />);
    const input = screen.getByRole<HTMLInputElement>('textbox');
    expect(input).toHaveValue('85,050');
    // The cursor is after "85,0". The user types a comma there and sees exactly that.
    await userEvent.type(input, ',', { initialSelectionStart: 4, initialSelectionEnd: 4 });
    expect(input).toHaveValue('85,0,50');
    expect(onValue).toHaveBeenLastCalledWith('85,0,50');
    expect(input.selectionStart).toBe('85,0,'.length);
  });

  it('takes the field’s own commas out of the value when the user edits the digits', async () => {
    const onValue = vi.fn();
    renderIn(language, <Harness control={MoneyInput} initial="12000" onValue={onValue} />);
    const input = screen.getByRole<HTMLInputElement>('textbox');
    expect(input).toHaveValue('12,000');
    await userEvent.type(input, '{Backspace}');
    expect(onValue).toHaveBeenLastCalledWith('1200');
    expect(input).toHaveValue('1,200');
    // A sum after grouped digits: the field's commas are not sent, the user's "+" is.
    await userEvent.type(input, '+5');
    expect(onValue).toHaveBeenLastCalledWith('1200+5');
    expect(input).toHaveValue('1200+5');
  });

  it('turns a comma the user takes away back into display grouping', () => {
    const onValue = vi.fn();
    renderIn(language, <Harness control={MoneyInput} initial="1,200" onValue={onValue} />);
    fireEvent.change(screen.getByRole('textbox'), { target: { value: '1200' } });
    expect(onValue).toHaveBeenLastCalledWith('1200');
    expect(screen.getByRole('textbox')).toHaveValue('1,200');
  });

  it('shows the backend’s ERR_NOT_A_NUMBER under the field and keeps the typed text', () => {
    const words = message(language, 'err-not-a-number');
    // It says what went wrong (letters, misplaced commas) and what to do (an example), NFR-USE-13.
    expect(words).toContain('12,000');
    expect(words).toMatch(language === 'ur' ? /کاما/ : /misplaced commas/);
    renderIn(
      language,
      <Field label={message(language, 'common-back')} error={words}>
        {(control) => <MoneyInput {...control} value="850,50" onChange={vi.fn()} />}
      </Field>,
    );
    const input = screen.getByRole('textbox');
    expect(input).toHaveValue('850,50');
    expect(input).toHaveAttribute('aria-invalid', 'true');
    expect(input).toHaveAttribute('dir', 'ltr');
    expect(screen.getByText(words).closest('p')).toHaveAttribute(
      'id',
      input.getAttribute('aria-describedby'),
    );
  });

  it('QtyInput groups like money', async () => {
    renderIn(language, <Harness control={QtyInput} onValue={vi.fn()} />);
    await userEvent.type(screen.getByRole('textbox'), '1000000');
    expect(screen.getByRole('textbox')).toHaveValue('1,000,000');
  });

  it('DateInput keeps DD-MM-YYYY text and the FR-7.19 shortcuts as typed', async () => {
    const onValue = vi.fn();
    renderIn(language, <Harness control={DateInput} onValue={onValue} />);
    const input = screen.getByRole('textbox');
    await userEvent.type(input, '10-10-2026');
    expect(input).toHaveValue('10-10-2026');
    expect(onValue).toHaveBeenLastCalledWith('10-10-2026');
    expect(input).toHaveAttribute('dir', 'ltr');
  });

  it('CodeInput never groups a code', async () => {
    const onValue = vi.fn();
    renderIn(language, <Harness control={CodeInput} onValue={onValue} />);
    const input = screen.getByRole('textbox');
    await userEvent.type(input, '0010');
    expect(input).toHaveValue('0010');
    expect(input).toHaveAttribute('inputmode', 'numeric');
  });

  it('TextInput sends what is typed and takes the field language', async () => {
    const onChange = vi.fn();
    renderIn(language, <TextInput value="" onChange={onChange} lang="en" />);
    await userEvent.type(screen.getByRole('textbox'), 'a');
    expect(onChange).toHaveBeenLastCalledWith('a');
    expect(screen.getByRole('textbox')).toHaveAttribute('lang', 'en');
    expect(screen.getByRole('textbox')).toHaveClass('type-field');
  });
});
