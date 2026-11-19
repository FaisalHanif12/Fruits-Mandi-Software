// Where a payable posting came from, in words (PRD FR-7.59, FR-16.11): a fruit credit on a seller
// says "پھل" with the پھل number, in both languages (plan 17d).

import { screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { SourceLabel } from '../../src/features/vouchers/components/SourceLabel';
import { LANGUAGES } from '../../src/i18n';
import { message, renderIn, visible } from '../helpers/render';

describe.each(LANGUAGES)('the source of a payable posting (%s)', (language) => {
  it('a fruit credit names the fruit and its number', () => {
    renderIn(language, <SourceLabel source="fruit" number={2} />);
    const words = message(language, 'vouchers-source-fruit', { number: 2 });
    expect(screen.getByText((content) => visible(content) === words)).toBeInTheDocument();
    expect(words).toContain('2');
    expect(words).toContain(message(language, 'glossary-phal'));
  });

  it('a source with no number leaves the number out', () => {
    renderIn(language, <SourceLabel source="fruit" number={null} />);
    expect(visible(document.body.textContent)).toBe(
      message(language, 'vouchers-source-fruit', { number: '' }),
    );
  });
});
