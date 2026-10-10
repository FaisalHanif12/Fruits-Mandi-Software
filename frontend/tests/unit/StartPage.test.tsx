// The first screen in both languages (plan 01 T3.7, Code Standards 9.1 "New screen").

import { act, fireEvent, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { appInfo } from '../../src/app/api';
import { StartPage } from '../../src/app/start/StartPage';
import { CommandError } from '../../src/shared/api';
import { useLanguageStore } from '../../src/shared/i18n';
import { renderRouted } from '../helpers/router';

vi.mock('../../src/app/api', () => ({ appInfo: vi.fn() }));
const appInfoMock = vi.mocked(appInfo);

/** Fluent wraps variables in bidi isolation marks; tests compare the visible text. */
function visible(text: string | null | undefined): string {
  return (text ?? '').replace(/[⁨⁩]/g, '');
}

/** The start page at "/", with a stub where the gallery link goes. */
function renderStartPage() {
  return renderRouted(
    { '/': StartPage, '/dev/gallery': () => <p data-testid="gallery-stub" /> },
    '/',
  );
}

/** Tauri puts this on the window; a plain browser tab does not have it. */
const BRIDGE = '__TAURI_INTERNALS__';
function setAppWindow(inApp: boolean) {
  if (inApp) Object.defineProperty(window, BRIDGE, { value: {}, configurable: true });
  else Reflect.deleteProperty(window, BRIDGE);
}

function pressAltL() {
  act(() => {
    fireEvent.keyDown(window, { code: 'KeyL', key: '¬', altKey: true });
  });
}

beforeEach(() => {
  setAppWindow(true);
  useLanguageStore.setState({ language: 'ur' });
  appInfoMock.mockResolvedValue({ version: '0.1.12', identifier: 'pk.mandisoftware.app' });
});

afterEach(() => {
  setAppWindow(false);
  vi.clearAllMocks();
});

describe('StartPage', () => {
  it('opens in Urdu, right to left, with the version from app_info', async () => {
    await renderStartPage();
    expect(document.documentElement.dir).toBe('rtl');
    expect(document.documentElement.lang).toBe('ur');
    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('منڈی سافٹ ویئر');
    const version = await screen.findByTestId('app-version');
    expect(visible(version.textContent)).toBe('ورژن 0.1.12');
    expect(version.querySelector('bdi[dir="ltr"]')).toHaveTextContent('0.1.12');
  });

  it('switches to English, left to right, with Alt+L and back again', async () => {
    await renderStartPage();
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
    await renderStartPage();
    await screen.findByTestId('app-version');
    fireEvent.click(screen.getByRole('button', { name: 'English' }));
    expect(document.documentElement.dir).toBe('ltr');
    expect(screen.getByRole('button', { name: 'English' })).toHaveAttribute('aria-pressed', 'true');
    fireEvent.click(screen.getByRole('button', { name: 'اردو' }));
    expect(document.documentElement.dir).toBe('rtl');
  });

  it('shows a calm loading line while the version is on its way', async () => {
    appInfoMock.mockReturnValue(new Promise(() => undefined));
    await renderStartPage();
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
    await renderStartPage();
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
    await renderStartPage();
    const alert = await screen.findByRole('alert');
    expect(alert).toHaveTextContent('کچھ غلط ہو گیا۔ دوبارہ کوشش کریں۔');
    expect(alert).not.toHaveTextContent('حوالہ');
  });

  it('inside the app window shows no browser note', async () => {
    await renderStartPage();
    await screen.findByTestId('app-version');
    expect(screen.queryByTestId('browser-preview-note')).toBeNull();
  });
});

// Development only: Umar opens the Vite page in a normal Chrome tab, where there is no Tauri bridge.
describe('StartPage in a plain browser tab (development)', () => {
  beforeEach(() => {
    setAppWindow(false);
  });

  it('shows a clear note instead of the error state, in Urdu and English', async () => {
    await renderStartPage();
    const note = await screen.findByTestId('browser-preview-note');
    expect(note).toHaveAttribute('role', 'note');
    expect(note).toHaveTextContent('براؤزر میں پیش نظارہ');
    expect(note).toHaveTextContent('ایپ سے رابطہ نہیں ہے');
    expect(screen.queryByRole('alert')).toBeNull();
    expect(screen.queryByRole('button', { name: 'دوبارہ کوشش کریں' })).toBeNull();
    expect(screen.queryByTestId('app-version')).toBeNull();
    // The page never asks the missing backend for anything.
    expect(appInfoMock).not.toHaveBeenCalled();

    pressAltL();
    expect(screen.getByTestId('browser-preview-note')).toHaveTextContent('Preview in a browser');
    expect(screen.getByTestId('browser-preview-note')).toHaveTextContent(
      'There is no app connection',
    );
    expect(screen.queryByRole('alert')).toBeNull();
  });
});

describe('StartPage link to the component gallery (development)', () => {
  it.each([
    ['ur', 'اجزاء کی گیلری'],
    ['en', 'Component gallery'],
  ] as const)('shows the link in %s and opens /dev/gallery', async (language, name) => {
    useLanguageStore.setState({ language });
    await renderStartPage();
    const link = await screen.findByRole('link', { name });
    expect(link).toHaveAttribute('href', '/dev/gallery');
    fireEvent.click(link);
    expect(await screen.findByTestId('gallery-stub')).toBeInTheDocument();
  });

  it('is there in a plain browser tab too, and follows the language switch', async () => {
    setAppWindow(false);
    await renderStartPage();
    expect(await screen.findByRole('link', { name: 'اجزاء کی گیلری' })).toBeInTheDocument();
    pressAltL();
    expect(screen.getByRole('link', { name: 'Component gallery' })).toBeInTheDocument();
  });
});
