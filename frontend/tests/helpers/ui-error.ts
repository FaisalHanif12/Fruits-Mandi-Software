// Test helper: a backend refusal read the way the app reads one, as JSON that is then checked.
// This lets a test use an error code that the generated types do not list yet (a new backend
// code, or an unknown one) without a type cast.

import type { UiError } from '../../src/bindings/UiError';
import { isUiError } from '../../src/shared/api';

export interface RefusalItem {
  code: string;
  field?: string | null;
  values?: Record<string, string | number>;
}

export const TEST_REFERENCE = 'E-20261010-0007';

export function refusal(items: readonly RefusalItem[], ref: string = TEST_REFERENCE): UiError {
  const json = JSON.stringify({
    ok: false,
    errors: items.map((item) => ({
      code: item.code,
      field: item.field ?? null,
      values: item.values ?? {},
    })),
    ref,
  });
  const value: unknown = JSON.parse(json);
  if (!isUiError(value)) throw new Error('test refusal is not a UiError');
  return value;
}
