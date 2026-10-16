// Robustness: MoneyInput and QtyInput send what the user typed (plan 02 T6, Code Standards M8,
// 4.2, CLAUDE.md rule 1, plan 02 contract 4.7). The backend parses the text and is the only judge
// of it, so the input must not repair, round, convert or drop anything the user typed.
//
// Rules checked:
// - Text with no comma is sent exactly as typed or pasted: Urdu digits, sums, decimals,
//   spaces, signs, invisible marks, letters and injection text all reach the backend unchanged.
// - Plain digits are shown grouped (display only, FR-7.20). The commas the field adds itself are
//   never sent: the user types 12000, sees 12,000, and "12000" is sent.
// - A comma the user typed or pasted is sent exactly as typed, in any place, and shown as typed.
//   "850,50" reaches the backend, which refuses it. "12,000", "1,00,000" and "1,316,040" are sent
//   as pasted too; the backend accepts those groupings. (Finding R02-6, fixed: the field used to
//   drop every comma, so "850,50" became 85,050 rupees.)

import { fireEvent, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { useGroupingStore, type Grouping } from '../../src/shared/format';
import { Field, MoneyInput, QtyInput } from '../../src/shared/ui';
import { renderIn } from '../helpers/render';

type Control = typeof MoneyInput;

function Harness({ control: Input, onValue }: { control: Control; onValue: (t: string) => void }) {
  const [value, setValue] = useState('');
  return (
    <Field label={<span />}>
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

const CONTROLS: readonly (readonly [string, Control])[] = [
  ['MoneyInput', MoneyInput],
  ['QtyInput', QtyInput],
];
const GROUPINGS: readonly Grouping[] = ['international', 'lakh'];

/** Pastes `text` into a fresh input and returns the value sent and the text shown. */
function paste(control: Control, text: string): { sent: string | undefined; shown: string } {
  const onValue = vi.fn<(t: string) => void>();
  const view = renderIn('ur', <Harness control={control} onValue={onValue} />);
  const input = screen.getByRole<HTMLInputElement>('textbox');
  fireEvent.change(input, { target: { value: text } });
  const result = { sent: onValue.mock.lastCall?.[0], shown: input.value };
  view.unmount();
  return result;
}

/** Texts with no comma. Each must be sent exactly. */
const NO_COMMA: readonly string[] = [
  '850.50',
  '850.',
  '.5',
  '20+25',
  '500-200',
  '-5',
  '+5',
  '۶۰۰',
  '٦٠٠',
  '١٬٢٠٠',
  '٨٥٠٫٥٠',
  '850۔50',
  '1،000',
  ' 600 ',
  '6 00',
  '6​00',
  '‏600',
  '‮12',
  '⁦600⁩',
  '６００',
  '1e5',
  '0x10',
  'NaN',
  'abc',
  "' OR 1=1 --",
  '<script>alert(1)</script>',
  '0000600',
  '9'.repeat(10_000),
];

afterEach(() => {
  useGroupingStore.setState({ grouping: 'international' });
});

describe.each(CONTROLS)('%s', (_name, control) => {
  describe.each(GROUPINGS)('with %s grouping', (grouping) => {
    it('sends text with no comma exactly as pasted', () => {
      useGroupingStore.setState({ grouping });
      const wrong: string[] = [];
      for (const text of NO_COMMA) {
        const { sent } = paste(control, text);
        if (sent !== text)
          wrong.push(
            `${JSON.stringify(text.slice(0, 20))} sent as ${JSON.stringify(sent?.slice(0, 20))}`,
          );
      }
      expect(wrong).toEqual([]);
    });

    it('shows only plain digits grouped; everything else as typed', () => {
      useGroupingStore.setState({ grouping });
      for (const text of NO_COMMA) {
        const { shown } = paste(control, text);
        if (/^[0-9]+$/.test(text)) {
          expect(shown.replaceAll(',', '')).toBe(text);
        } else {
          expect(shown).toBe(text);
        }
      }
    });

    it('sends commas in valid group places exactly as pasted (the backend accepts them)', () => {
      useGroupingStore.setState({ grouping });
      for (const text of ['12,000', '1,00,000', '1,316,040', '10,00,00,00,000']) {
        expect(paste(control, text)).toEqual({ sent: text, shown: text });
      }
    });

    // Finding R02-6, fixed: a comma the user pasted is never dropped.
    it('sends a comma in an odd place unchanged, so the backend refuses it', () => {
      useGroupingStore.setState({ grouping });
      for (const text of ['850,50', '1,00,0', '1,2', '12,34,5', ',100', '100,', '1,,000']) {
        expect(paste(control, text).sent).toBe(text);
      }
    });
  });

  it('typing key by key sends each step exactly (Urdu digits and a sum)', async () => {
    const onValue = vi.fn<(t: string) => void>();
    renderIn('en', <Harness control={control} onValue={onValue} />);
    const input = screen.getByRole('textbox');
    await userEvent.type(input, '۲۰+۲۵');
    expect(onValue.mock.calls.map((c) => c[0])).toEqual(['۲', '۲۰', '۲۰+', '۲۰+۲', '۲۰+۲۵']);
    expect(input).toHaveValue('۲۰+۲۵');
  });
});

describe('finding R02-6, the money effect (fixed)', () => {
  it('a comma typed as a decimal point is sent as typed, never as a 100 times larger amount', () => {
    // The munshi means 850 rupees 50 paisa and types a comma for the point (common on phones
    // and in other countries). The backend answers ERR_NOT_A_NUMBER for "850,50"; the field
    // must send exactly that text and show it, not "85050" / "85,050".
    expect(paste(MoneyInput, '850,50')).toEqual({ sent: '850,50', shown: '850,50' });
    expect(paste(QtyInput, '850,50')).toEqual({ sent: '850,50', shown: '850,50' });
  });
});

describe.each(CONTROLS)('%s typed key by key', (_name, control) => {
  it('never sends the commas the field adds for display', async () => {
    const onValue = vi.fn<(t: string) => void>();
    renderIn('en', <Harness control={control} onValue={onValue} />);
    const input = screen.getByRole<HTMLInputElement>('textbox');
    await userEvent.type(input, '1316040');
    expect(onValue.mock.calls.map((c) => c[0])).toEqual([
      '1',
      '13',
      '131',
      '1316',
      '13160',
      '131604',
      '1316040',
    ]);
    expect(input.value).toBe('1,316,040');
    // One more digit in front of the grouped text: still no field comma sent.
    await userEvent.type(input, '5');
    expect(onValue.mock.lastCall?.[0]).toBe('13160405');
    // A backspace over grouped text: same.
    await userEvent.type(input, '{Backspace}{Backspace}');
    expect(onValue.mock.lastCall?.[0]).toBe('131604');
  });

  it('keeps a comma the user types, and every digit after it', async () => {
    const onValue = vi.fn<(t: string) => void>();
    renderIn('en', <Harness control={control} onValue={onValue} />);
    const input = screen.getByRole<HTMLInputElement>('textbox');
    await userEvent.type(input, '850,50');
    expect(onValue.mock.calls.map((c) => c[0])).toEqual([
      '8',
      '85',
      '850',
      '850,',
      '850,5',
      '850,50',
    ]);
    expect(input.value).toBe('850,50');
  });

  it('keeps a comma typed after grouped digits, with the field commas around it as shown', async () => {
    const onValue = vi.fn<(t: string) => void>();
    renderIn('en', <Harness control={control} onValue={onValue} />);
    const input = screen.getByRole<HTMLInputElement>('textbox');
    await userEvent.type(input, '12000');
    expect(input.value).toBe('12,000');
    await userEvent.type(input, ',');
    // What is sent is what is on the screen. "12,000," is refused by the backend.
    expect(onValue.mock.lastCall?.[0]).toBe('12,000,');
    expect(input.value).toBe('12,000,');
  });

  it('shows the amount that is sent after the user deletes the comma they typed', async () => {
    const onValue = vi.fn<(t: string) => void>();
    renderIn('en', <Harness control={control} onValue={onValue} />);
    const input = screen.getByRole<HTMLInputElement>('textbox');
    await userEvent.type(input, '850,50');
    await userEvent.type(input, '{ArrowLeft}{ArrowLeft}{Backspace}');
    expect(onValue.mock.lastCall?.[0]).toBe('85050');
    // The screen shows the same amount that goes to the backend.
    expect(input.value.replaceAll(',', '')).toBe('85050');
  });
});
