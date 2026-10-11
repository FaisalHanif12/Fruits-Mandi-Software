// DataCheckFailedState: the window shown when the start-up data check fails (plan 03 contract
// 4.3, NFR-REL-7), in Urdu and English.

import { fireEvent, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import { DataCheckFailedState } from '../../src/shared/ui';
import { message, renderIn, visible } from '../helpers/render';
import { TEST_REFERENCE } from '../helpers/ui-error';

describe.each(LANGUAGES)('DataCheckFailedState (%s)', (language) => {
  it('says what happened and what to do, and shows the reference left to right', () => {
    renderIn(language, <DataCheckFailedState reference={TEST_REFERENCE} onRetry={vi.fn()} />);
    const alert = screen.getByRole('alert');
    const text = visible(alert.textContent);
    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent(
      message(language, 'common-data-check-title'),
    );
    expect(text).toContain(message(language, 'err-data-check-failed'));
    expect(text).toContain(TEST_REFERENCE);
    expect(alert.querySelector('bdi[dir="ltr"]')).toHaveTextContent(TEST_REFERENCE);
    expect(text).not.toContain('ERR_');
    expect(text).not.toContain('err-');
  });

  it('has one button, focused at once, that runs the check again with Enter or a click', async () => {
    const onRetry = vi.fn();
    renderIn(language, <DataCheckFailedState reference={TEST_REFERENCE} onRetry={onRetry} />);
    const button = screen.getByRole('button', { name: message(language, 'common-retry') });
    expect(screen.getAllByRole('button')).toHaveLength(1);
    expect(button).toHaveFocus();
    await userEvent.keyboard('{Enter}');
    expect(onRetry).toHaveBeenCalledTimes(1);
    fireEvent.click(button);
    expect(onRetry).toHaveBeenCalledTimes(2);
  });

  it('waits while the check runs, and cannot be pressed twice', () => {
    const onRetry = vi.fn();
    renderIn(
      language,
      <DataCheckFailedState reference={TEST_REFERENCE} onRetry={onRetry} retrying />,
    );
    const button = screen.getByRole('button');
    expect(button).toBeDisabled();
    expect(button).toHaveAttribute('aria-busy', 'true');
    fireEvent.click(button);
    expect(onRetry).not.toHaveBeenCalled();
  });

  it('shows no reference line when there is none', () => {
    renderIn(language, <DataCheckFailedState reference="" onRetry={vi.fn()} />);
    expect(visible(screen.getByRole('alert').textContent)).not.toContain('E-');
  });

  it('fills the window when alone, and sits inside a card when not', () => {
    const { container, unmount } = renderIn(
      language,
      <DataCheckFailedState reference={TEST_REFERENCE} onRetry={vi.fn()} />,
    );
    expect(container.querySelector('.min-h-screen')).not.toBeNull();
    unmount();
    const inCard = renderIn(
      language,
      <DataCheckFailedState reference={TEST_REFERENCE} onRetry={vi.fn()} fullWindow={false} />,
    );
    expect(inCard.container.querySelector('.min-h-screen')).toBeNull();
    expect(screen.getByRole('heading', { level: 2 })).toBeInTheDocument();
    expect(screen.getByRole('button')).not.toHaveFocus();
  });
});
