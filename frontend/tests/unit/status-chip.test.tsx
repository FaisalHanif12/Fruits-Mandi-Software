// The state of a row as a chip (plan 75, item 4): the word is always in it, with an icon, in a soft
// tone; a list column draws it only when it asks (`chip`), centred under its heading, and a column that
// does not ask is drawn as plain text, as before. Colour is never the only signal (NFR-USE-12).

import { render, screen } from '@testing-library/react';
import { LocalizationProvider, ReactLocalization } from '@fluent/react';
import { Ban, Check } from 'lucide-react';
import { describe, expect, it } from 'vitest';
import { LANGUAGES, bundles } from '../../src/i18n';
import { ListGrid, type ListColumn } from '../../src/shared/grid';
import {
  STATUS_CANCELLED,
  STATUS_CLOSED,
  STATUS_OPEN,
  STATUS_SAVED,
  StatusChip,
} from '../../src/shared/ui';
import { message, renderIn } from '../helpers/render';

describe.each(LANGUAGES)('StatusChip (%s)', (language) => {
  it('shows the word with an icon that a screen reader skips', () => {
    renderIn(
      language,
      <StatusChip tone="warn" icon={Ban}>
        {message(language, 'bikri-status-cancelled')}
      </StatusChip>,
    );
    const word = screen.getByText(message(language, 'bikri-status-cancelled'));
    const chip = word.parentElement;
    expect(chip?.querySelector('svg')).toHaveAttribute('aria-hidden', 'true');
    // The word is in the chip as text, never only a colour or an icon.
    expect(chip).toHaveTextContent(message(language, 'bikri-status-cancelled'));
    expect(chip).toHaveClass('bg-warn-soft', 'text-warn');
  });

  it('never grows past its cell: the word is cut with an ellipsis, not the chip', () => {
    renderIn(
      language,
      <StatusChip tone="primary" icon={Check}>
        {message(language, 'bikri-status-saved')}
      </StatusChip>,
    );
    const word = screen.getByText(message(language, 'bikri-status-saved'));
    expect(word).toHaveClass('truncate');
    expect(word.parentElement).toHaveClass('max-w-full');
  });
});

describe('the shared looks', () => {
  it('give each state its own icon and tone: saved and open in order, cancelled amber, closed quiet', () => {
    expect(STATUS_SAVED.tone).toBe('primary');
    expect(STATUS_OPEN.tone).toBe('primary');
    expect(STATUS_CANCELLED.tone).toBe('warn');
    expect(STATUS_CLOSED.tone).toBe('muted');
    const icons = new Set([
      STATUS_SAVED.icon,
      STATUS_CANCELLED.icon,
      STATUS_OPEN.icon,
      STATUS_CLOSED.icon,
    ]);
    // A state is never told by colour alone: no two states share an icon.
    expect(icons.size).toBe(4);
  });
});

interface Row {
  id: string;
  state: 'saved' | 'cancelled' | 'none';
}

const ROWS: Row[] = [
  { id: '1', state: 'saved' },
  { id: '2', state: 'cancelled' },
  { id: '3', state: 'none' },
];

const WORDS: Record<Row['state'], string> = { saved: 'Saved', cancelled: 'Cancelled', none: '' };

function grid(withChip: boolean) {
  const columns: ListColumn<Row>[] = [
    {
      field: 'state',
      type: 'status',
      headerId: 'common-back',
      value: (row) => WORDS[row.state],
      ...(withChip
        ? { chip: (row: Row) => (row.state === 'cancelled' ? STATUS_CANCELLED : STATUS_SAVED) }
        : {}),
    },
  ];
  return render(
    <LocalizationProvider l10n={new ReactLocalization([bundles.en])}>
      <ListGrid<Row>
        columns={columns}
        rows={ROWS}
        getRowId={(row) => row.id}
        labelId="common-back"
        heightMode="auto"
        onOpen={() => undefined}
      />
    </LocalizationProvider>,
  );
}

describe('a list column with `chip`', () => {
  it('draws the word of each row as a chip, centred under a centred heading', async () => {
    grid(true);
    const saved = await screen.findByText('Saved');
    expect(saved.parentElement).toHaveClass('bg-primary-soft');
    expect(screen.getByText('Cancelled').parentElement).toHaveClass('bg-warn-soft');
    const cell = saved.closest('.ag-cell');
    expect(cell).toHaveClass('cell-center', 'cell-chip');
    expect(document.querySelector('.ag-header-cell.header-center')).not.toBeNull();
  });

  it('draws a row with no word as an empty cell, not an empty chip', async () => {
    grid(true);
    await screen.findByText('Saved');
    const empty = document.querySelector('.ag-row[row-id="3"] [col-id="state"]');
    expect(empty?.querySelector('svg')).toBeNull();
    expect(empty?.textContent).toBe('');
  });

  it('a column without `chip` stays plain text', async () => {
    grid(false);
    const saved = await screen.findByText('Saved');
    expect(saved.closest('.ag-cell')).not.toHaveClass('cell-chip');
    expect(saved.parentElement?.querySelector('svg')).toBeNull();
    expect(document.querySelector('.ag-header-cell.header-center')).toBeNull();
  });
});
