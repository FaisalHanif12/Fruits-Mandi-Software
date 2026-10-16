import { describe, expect, it } from 'vitest';
import {
  CommandError,
  errorMessageId,
  isUiError,
  toUiError,
  uiErrorOf,
} from '../../src/shared/api';

describe('shared/api', () => {
  it('maps error codes to Fluent IDs', () => {
    expect(errorMessageId('ERR_NOT_IMPLEMENTED')).toBe('err-not-implemented');
    expect(errorMessageId('ERR_INTERNAL')).toBe('err-internal');
  });

  it('keeps a backend UiError as it is', () => {
    const uiError = {
      ok: false as const,
      errors: [{ code: 'ERR_NOT_IMPLEMENTED' as const, field: null, values: {} }],
      ref: 'E-20261010-0001',
    };
    expect(isUiError(uiError)).toBe(true);
    expect(toUiError(uiError)).toBe(uiError);
    expect(uiErrorOf(new CommandError(uiError))).toBe(uiError);
  });

  it('turns anything else into ERR_INTERNAL, so the screen always has a message', () => {
    for (const thrown of [
      'window.__TAURI_INTERNALS__ is undefined',
      new Error('x'),
      null,
      { ok: true },
    ]) {
      const uiError = toUiError(thrown);
      expect(uiError.errors.map((item) => item.code)).toEqual(['ERR_INTERNAL']);
      expect(uiError.ref).toBe('');
    }
  });
});
