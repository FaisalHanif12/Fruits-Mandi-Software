// The data check gate (plan 03 Section 4.3, NFR-REL-7): the app waits for the start-up check,
// shows DataCheckFailedState with the reference when it fails, and opens the screens when it
// passes. Each in Urdu and English.

import { fireEvent, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { appDataStatus } from '../../src/app/api';
import { DataCheckGate } from '../../src/app/data-check/DataCheckGate';
import { LANGUAGES } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import { useLanguageStore } from '../../src/shared/i18n';
import { message, visible } from '../helpers/render';
import { renderRouted } from '../helpers/router';
import { TEST_REFERENCE, refusal } from '../helpers/ui-error';

vi.mock('../../src/app/api', () => ({ appDataStatus: vi.fn() }));
const statusMock = vi.mocked(appDataStatus);

function Gated() {
  return (
    <DataCheckGate>
      <p data-testid="screens" />
    </DataCheckGate>
  );
}

afterEach(() => {
  vi.clearAllMocks();
  useLanguageStore.setState({ language: 'ur' });
});

describe.each(LANGUAGES)('DataCheckGate (%s)', (language) => {
  beforeEach(() => {
    useLanguageStore.setState({ language });
  });

  it('a failed check fills the window with the message and the reference, and hides the screens', async () => {
    statusMock.mockResolvedValue({ state: 'failed', reference: TEST_REFERENCE });
    await renderRouted({ '/': Gated }, '/');
    const alert = await screen.findByRole('alert');
    const text = visible(alert.textContent);
    expect(text).toContain(message(language, 'common-data-check-title'));
    expect(text).toContain(message(language, 'err-data-check-failed'));
    expect(alert.querySelector('bdi[dir="ltr"]')).toHaveTextContent(TEST_REFERENCE);
    expect(screen.queryByTestId('screens')).not.toBeInTheDocument();
  });

  it('Try again asks for the status once more, and opens the screens when it is fine', async () => {
    statusMock.mockResolvedValueOnce({ state: 'failed', reference: TEST_REFERENCE });
    statusMock.mockResolvedValue({ state: 'ok', reference: null });
    await renderRouted({ '/': Gated }, '/');
    fireEvent.click(await screen.findByRole('button', { name: message(language, 'common-retry') }));
    expect(await screen.findByTestId('screens')).toBeInTheDocument();
    expect(statusMock).toHaveBeenCalledTimes(2);
  });

  it('waits while the check runs, asks again, and opens the screens when it passes', async () => {
    statusMock.mockResolvedValueOnce({ state: 'checking', reference: null });
    statusMock.mockResolvedValue({ state: 'ok', reference: null });
    await renderRouted({ '/': Gated }, '/');
    expect(screen.queryByTestId('screens')).not.toBeInTheDocument();
    expect(screen.getByRole('status')).toHaveAttribute('aria-busy', 'true');
    await waitFor(() => {
      expect(screen.getByTestId('screens')).toBeInTheDocument();
    });
    expect(statusMock).toHaveBeenCalledTimes(2);
  });

  it('a passed check shows the screens at once and never asks again', async () => {
    statusMock.mockResolvedValue({ state: 'ok', reference: null });
    await renderRouted({ '/': Gated }, '/');
    expect(await screen.findByTestId('screens')).toBeInTheDocument();
    await new Promise((resolve) => setTimeout(resolve, 450));
    expect(statusMock).toHaveBeenCalledTimes(1);
  });

  it('a failed check with no reference still shows the message, without a reference line', async () => {
    statusMock.mockResolvedValue({ state: 'failed', reference: null });
    await renderRouted({ '/': Gated }, '/');
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'err-data-check-failed'));
    expect(alert.querySelector('bdi[dir="ltr"]')).toBeNull();
  });

  it('if the status itself cannot be read, shows the error with its reference and a retry', async () => {
    statusMock.mockRejectedValueOnce(new CommandError(refusal([{ code: 'ERR_INTERNAL' }])));
    statusMock.mockResolvedValue({ state: 'ok', reference: null });
    await renderRouted({ '/': Gated }, '/');
    const alert = await screen.findByRole('alert');
    const text = visible(alert.textContent);
    expect(text).toContain(message(language, 'err-internal'));
    expect(alert.querySelector('bdi[dir="ltr"]')).toHaveTextContent(TEST_REFERENCE);
    fireEvent.click(screen.getByRole('button', { name: message(language, 'common-retry') }));
    expect(await screen.findByTestId('screens')).toBeInTheDocument();
  });
});
