// The look of a dashboard tile and of a figure that heads a screen (plan 75, items 1 and 7): a tile can
// carry a soft icon chip that decorates and takes the tone of its figure; a `Fact` can be a small stat
// card with the figure style. Neither changes what is said: the label, the figure and the note stay as
// text, and the whole tile is still one button (NFR-USE-12).

import { Localized } from '@fluent/react';
import { screen } from '@testing-library/react';
import { Wallet } from 'lucide-react';
import { describe, expect, it, vi } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import { Fact, Num, Tile } from '../../src/shared/ui';
import { renderIn } from '../helpers/render';

describe.each(LANGUAGES)('Tile with an icon (%s)', (language) => {
  it('draws a decorative chip that a screen reader skips, and keeps the tile one button', () => {
    renderIn(
      language,
      <Tile label={<Localized id="common-back" />} icon={Wallet} onOpen={vi.fn()}>
        <Num>{'12,000'}</Num>
      </Tile>,
    );
    const tile = screen.getByRole('button');
    const icon = tile.querySelector('svg');
    expect(icon).toHaveAttribute('aria-hidden', 'true');
    expect(icon?.parentElement).toHaveClass('bg-primary-soft', 'text-primary');
    expect(tile).toHaveTextContent('12,000');
  });

  it('gives the chip the tone of the figure, so an amber figure has an amber chip', () => {
    renderIn(
      language,
      <Tile label={<Localized id="common-back" />} icon={Wallet} tone="warn" onOpen={vi.fn()}>
        <Num>{'5'}</Num>
      </Tile>,
    );
    expect(screen.getByRole('button').querySelector('svg')?.parentElement).toHaveClass(
      'bg-warn-soft',
      'text-warn',
    );
  });

  it('draws no chip when it has no icon', () => {
    renderIn(
      language,
      <Tile label={<Localized id="common-back" />} onOpen={vi.fn()}>
        <Num>{'5'}</Num>
      </Tile>,
    );
    expect(screen.getByRole('button').querySelector('svg')).toBeNull();
  });
});

describe('Fact as a stat card', () => {
  it('is a small card with the figure style, and a plain Fact is unchanged', () => {
    renderIn(
      'en',
      <dl>
        <Fact stat label={<Localized id="common-back" />}>
          <Num>{'50,000'}</Num>
        </Fact>
        <Fact label={<Localized id="common-retry" />}>
          <Num>{'1'}</Num>
        </Fact>
      </dl>,
    );
    const stat = screen.getByText('50,000').closest('dd');
    expect(stat).toHaveClass('type-figure');
    expect(stat?.parentElement).toHaveClass('rounded-2', 'border', 'bg-surface');
    const plain = screen.getByText('1').closest('dd');
    expect(plain).toHaveClass('type-field');
    expect(plain?.parentElement).not.toHaveClass('border');
  });
});
