// The first screen in both languages (plan 01 T3.7, Code Standards 9.1 "New screen").

import { act, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { appInfo } from '../../src/app/api';
import { AppProviders, createQueryClient } from '../../src/app/providers';
import { StartPage } from '../../src/app/start/StartPage';
import { CommandError } from '../../src/shared/api';
import { useLanguageStore } from '../../src/shared/i18n';

vi.mock('../../src/app/api', () => ({ appInfo: vi.fn() }));
const appInfoMock = vi.mocked(appInfo);

/** Fluent wraps variables in bidi isolation marks; tests compare the visible text. */
function visible(text: string | null | undefined): string {
  return (text ?? '').replace(/[⁨⁩]/g, '');
}

function renderStartPage() {
  return render(
    <AppProviders queryClient={createQueryClient()}>
      <StartPage />
    </AppProviders>,
  );
}

function pressAltL() {
  act(() => {
    fireEvent.keyDown(window, { code: 'KeyL', key: '¬', altKey: true });
  });
}

beforeEach(() => {
  useLanguageStore.setState({ language: 'ur' });
  appInfoMock.mockResolvedValue({ version: '0.1.12', identifier: 'pk.mandisoftware.app' });
});

afterEach(() => {
  vi.clearAllMocks();
});

describe('StartPage', () => {
  it('opens in Urdu, right to left, with the version from app_info', async () => {
    renderStartPage();
    expect(document.documentElement.dir).toBe('rtl');
    expect(document.documentElement.lang).toBe('ur');
    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('منڈی سافٹ ویئر');
    const version = await screen.findByTestId('app-version');
    expect(visible(version.textContent)).toBe('ورژن 0.1.12');
    expect(version.querySelector('bdi[dir="ltr"]')).toHaveTextContent('0.1.12');
  });

  it('switches to English, left to right, with Alt+L and back again', async () => {
    renderStartPage();
    await screen.findByTestId('app-version');

    pressAltL();
    expect(document.documentElement.dir).toBe('ltr');
    expect(document.documentElement.lang).toBe('en');
    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Mandi Software');
    expect(visible(screen.getByTestId('app-version').textContent)).toBe('Version 0.1.12');
    expect(document.title).toBe('Mandi Software');

    pressAltL();
    expect(document.documentElement.dir).toBe('rtl');
    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('منڈی سافٹ ویئر');
  });

  it('switches with the buttons too, never only with the key (PRD FR-7.3)', async () => {
    renderStartPage();
    await screen.findByTestId('app-version');
    fireEvent.click(screen.getByRole('button', { name: 'English' }));
    expect(document.documentElement.dir).toBe('ltr');
    expect(screen.getByRole('button', { name: 'English' })).toHaveAttribute('aria-pressed', 'true');
    fireEvent.click(screen.getByRole('button', { name: 'اردو' }));
    expect(document.documentElement.dir).toBe('rtl');
  });

  it('shows a calm loading line while the version is on its way', () => {
    appInfoMock.mockReturnValue(new Promise(() => undefined));
    renderStartPage();
    expect(screen.getByText('ذرا انتظار کریں…')).toBeInTheDocument();
  });

  it('shows the error in the app language with its reference, never the raw code', async () => {
    appInfoMock.mockRejectedValue(
      new CommandError({
        ok: false,
        errors: [{ code: 'ERR_NOT_IMPLEMENTED', field: null, values: {} }],
        ref: 'E-20261010-0001',
      }),
    );
    renderStartPage();
    const alert = await screen.findByRole('alert');
    expect(alert).toHaveTextContent('یہ حصہ ابھی تیار نہیں۔ یہ اگلے ورژن میں آئے گا۔');
    expect(alert).toHaveTextContent('E-20261010-0001');
    expect(alert).not.toHaveTextContent('ERR_NOT_IMPLEMENTED');
    expect(screen.getByRole('button', { name: 'دوبارہ کوشش کریں' })).toBeInTheDocument();

    pressAltL();
    expect(screen.getByRole('alert')).toHaveTextContent('This part is not ready yet.');
  });

  it('shows the general error when the app bridge is missing', async () => {
    appInfoMock.mockRejectedValue(
      new CommandError({
        ok: false,
        errors: [{ code: 'ERR_INTERNAL', field: null, values: {} }],
        ref: '',
      }),
    );
    renderStartPage();
    const alert = await screen.findByRole('alert');
    expect(alert).toHaveTextContent('کچھ غلط ہو گیا۔ دوبارہ کوشش کریں۔');
    expect(alert).not.toHaveTextContent('حوالہ');
  });
});
