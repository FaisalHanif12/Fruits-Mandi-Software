// Card, the four-state pieces, Num and Button states, in Urdu and English (plan 02 T3,
// design system 6, Code Standards F8, F25).

import { act, fireEvent, screen } from '@testing-library/react';
import { Users } from 'lucide-react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { UiError } from '../../src/bindings/UiError';
import { LANGUAGES } from '../../src/i18n';
import { useGroupingStore } from '../../src/shared/format';
import {
  Button,
  Card,
  Code,
  EmptyState,
  ErrorState,
  LOADING_SKELETON_DELAY_MS,
  LoadingState,
  Num,
} from '../../src/shared/ui';
import { paisaForTest } from '../helpers/paisa';
import { message, renderIn, visible } from '../helpers/render';

afterEach(() => {
  useGroupingStore.setState({ grouping: 'international' });
  vi.useRealTimers();
});

describe.each(LANGUAGES)('display pieces (%s)', (language) => {
  it('Card has a white ground, a border, 12 px radius and an optional title', () => {
    renderIn(language, <Card title={message(language, 'common-back')}>{null}</Card>);
    const card = screen.getByRole('heading', { level: 2 }).closest('section');
    expect(card).toHaveClass('bg-surface', 'border', 'border-border', 'rounded-3', 'p-5');
    expect(screen.getByRole('heading')).toHaveTextContent(message(language, 'common-back'));
  });

  it('EmptyState says why, what to do, and offers one button (FR-7.46)', () => {
    const onClick = vi.fn();
    renderIn(
      language,
      <EmptyState
        icon={Users}
        title={message(language, 'common-loading')}
        help={message(language, 'common-retry')}
        action={
          <Button variant="primary" onClick={onClick}>
            {message(language, 'common-back')}
          </Button>
        }
      />,
    );
    expect(screen.getByRole('heading')).toHaveTextContent(message(language, 'common-loading'));
    expect(screen.getByText(message(language, 'common-retry'))).toHaveClass('text-ink-muted');
    fireEvent.click(screen.getByRole('button'));
    expect(onClick).toHaveBeenCalledOnce();
  });

  it('ErrorState shows plain words, the reference number left to right, and retries', () => {
    const onRetry = vi.fn();
    const error: UiError = {
      ok: false,
      errors: [{ code: 'ERR_INTERNAL', field: null, values: {} }],
      ref: 'E-20261010-0007',
    };
    renderIn(
      language,
      <ErrorState title={message(language, 'common-back')} error={error} onRetry={onRetry} />,
    );
    const alert = screen.getByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'err-internal'));
    expect(alert.textContent).not.toContain('ERR_');
    expect(alert.querySelector('bdi[dir="ltr"]')).toHaveTextContent('E-20261010-0007');
    fireEvent.click(screen.getByRole('button', { name: message(language, 'common-retry') }));
    expect(onRetry).toHaveBeenCalledOnce();
  });

  it('groups a limit sent with an error, the same as on screen', () => {
    const error: UiError = {
      ok: false,
      errors: [{ code: 'ERR_QTY_TOO_LARGE', field: 'qty', values: { max: 1_000_000 } }],
      ref: '',
    };
    renderIn(
      language,
      <ErrorState title={message(language, 'common-back')} error={error} onRetry={vi.fn()} />,
    );
    const text = visible(screen.getByRole('alert').textContent);
    expect(text).toContain(message(language, 'err-qty-too-large', { max: '1,000,000' }));
  });

  it('LoadingState shows nothing for 300 ms, then skeleton rows (no flash)', () => {
    vi.useFakeTimers();
    renderIn(language, <LoadingState />);
    const status = screen.getByRole('status');
    expect(status).toHaveAttribute('aria-busy', 'true');
    expect(status).toHaveTextContent(message(language, 'common-loading'));
    expect(screen.queryAllByTestId('skeleton-row')).toHaveLength(0);
    act(() => {
      vi.advanceTimersByTime(LOADING_SKELETON_DELAY_MS - 1);
    });
    expect(screen.queryAllByTestId('skeleton-row')).toHaveLength(0);
    act(() => {
      vi.advanceTimersByTime(1);
    });
    expect(screen.getAllByTestId('skeleton-row')).toHaveLength(4);
  });

  it('Num shows an amount from paisa, isolated, left to right, tabular', () => {
    renderIn(language, <Num paisa={paisaForTest(131_604_000)} />);
    const num = screen.getByText('1,316,040');
    expect(num.tagName).toBe('BDI');
    expect(num).toHaveAttribute('dir', 'ltr');
    expect(num).toHaveClass('type-number');
  });

  it('Num follows the lakh grouping setting', () => {
    useGroupingStore.setState({ grouping: 'lakh' });
    renderIn(language, <Num paisa={paisaForTest(131_604_000)} />);
    expect(screen.getByText('13,16,040')).toBeInTheDocument();
  });

  it('Num and Code isolate text they are given', () => {
    renderIn(
      language,
      <p>
        <Num>
          <span data-testid="qty" />
        </Num>
        <Code>
          <span data-testid="truck" />
        </Code>
      </p>,
    );
    expect(screen.getByTestId('qty').parentElement).toHaveAttribute('dir', 'ltr');
    expect(screen.getByTestId('truck').parentElement?.tagName).toBe('BDI');
  });

  it('Button: busy and disabled cannot be pressed again', () => {
    const onClick = vi.fn();
    renderIn(
      language,
      <>
        <Button busy onClick={onClick}>
          <span />
        </Button>
        <Button disabled onClick={onClick}>
          <span />
        </Button>
      </>,
    );
    const [busy, disabled] = screen.getAllByRole('button');
    expect(busy).toHaveAttribute('aria-busy', 'true');
    expect(busy).toBeDisabled();
    expect(disabled).toBeDisabled();
    if (busy !== undefined) fireEvent.click(busy);
    expect(onClick).not.toHaveBeenCalled();
  });
});
