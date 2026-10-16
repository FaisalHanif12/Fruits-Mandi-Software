// Shared buttons from the design system (design system 6, PRD FR-7.4).

import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { SHORTCUTS } from '../../src/shared/keyboard';
import { Button, SegmentedControl } from '../../src/shared/ui';

describe('Button', () => {
  it('shows its key in a chip after the label, left to right', () => {
    const onClick = vi.fn();
    render(
      <Button variant="primary" shortcut={SHORTCUTS.languageToggle} onClick={onClick}>
        <span data-testid="label" />
      </Button>,
    );
    const button = screen.getByRole('button');
    const chip = button.querySelector('kbd');
    expect(chip).toHaveTextContent('Alt+L');
    expect(chip).toHaveAttribute('dir', 'ltr');
    expect(chip).toHaveClass('bg-primary-chip');
    expect(button).toHaveClass('h-control', 'rounded-2', 'bg-primary');
    fireEvent.click(button);
    expect(onClick).toHaveBeenCalledOnce();
  });

  it('uses the light chip on a secondary button', () => {
    render(
      <Button shortcut={SHORTCUTS.languageToggle} onClick={vi.fn()}>
        <span />
      </Button>,
    );
    expect(screen.getByRole('button')).toHaveClass('border-border-control');
    expect(screen.getByRole('button').querySelector('kbd')).toHaveClass('bg-surface-sunken');
  });
});

describe('SegmentedControl', () => {
  it('marks exactly one choice as pressed and reports the new choice', () => {
    const onChange = vi.fn();
    render(
      <SegmentedControl
        groupLabel="1"
        value="a"
        onChange={onChange}
        options={[
          { value: 'a', label: <span data-testid="a" /> },
          { value: 'b', label: <span data-testid="b" />, lang: 'en' },
        ]}
      />,
    );
    const [first, second] = screen.getAllByRole('button');
    expect(first).toHaveAttribute('aria-pressed', 'true');
    expect(second).toHaveAttribute('aria-pressed', 'false');
    expect(second).toHaveAttribute('lang', 'en');
    if (second !== undefined) fireEvent.click(second);
    expect(onChange).toHaveBeenCalledWith('b');
  });
});
