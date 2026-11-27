// A picker that is the last field of a form passes its Enter to the form, which finishes (plan 46,
// PRD FR-7.14). The value the picker has just put in must be in the save: the form must not finish
// with the values it had before the pick (plan 99: the mark form saved without its owner when the
// list had answered before Enter).

import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState, type KeyboardEvent } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { passEnterToForm, useFormNavigation } from '../../src/shared/keyboard';

function Harness({ onSave }: { onSave: (text: string, picked: string) => void }) {
  const [text, setText] = useState('');
  const [picked, setPicked] = useState('');
  const navigation = useFormNavigation(() => {
    onSave(text, picked);
  });
  return (
    <form
      {...navigation}
      onSubmit={(event) => {
        event.preventDefault();
      }}
    >
      <input
        value={text}
        onChange={(event) => {
          setText(event.target.value);
        }}
      />
      <input
        value={picked}
        onChange={(event) => {
          setPicked(event.target.value);
        }}
        onKeyDown={(event: KeyboardEvent<HTMLInputElement>) => {
          if (event.key !== 'Enter') return;
          // What PickerBase does on the last field: the pick, then the Enter goes to the form.
          event.preventDefault();
          setPicked('Rahim');
          passEnterToForm(event);
        }}
      />
    </form>
  );
}

describe('Enter that picks on the last field of a form', () => {
  it('finishes the form once, with the picked value', async () => {
    const onSave = vi.fn();
    render(<Harness onSave={onSave} />);
    const user = userEvent.setup();
    const [text, picker] = screen.getAllByRole('textbox');
    if (text === undefined || picker === undefined) throw new Error('two boxes');
    await user.type(text, 'AR');
    await user.type(picker, 'rah');
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(onSave).toHaveBeenCalled();
    });
    expect(onSave.mock.calls).toEqual([['AR', 'Rahim']]);
  });

  it('an Enter in a text field on the last field still finishes the form at once', async () => {
    const onSave = vi.fn();
    function Plain() {
      const navigation = useFormNavigation(onSave);
      return (
        <form {...navigation}>
          <input />
        </form>
      );
    }
    render(<Plain />);
    const user = userEvent.setup();
    await user.type(screen.getByRole('textbox'), 'a{Enter}');
    expect(onSave).toHaveBeenCalledTimes(1);
  });
});
