// Robustness: a name that is really markup or a script is shown as plain text and runs nothing
// (Code Standards V0, S5; NFR-SEC-10; the injection items of the robustness list). Owner:
// `robustness`.
//
// The backend refuses control characters and wrong-script names, but it stores `<`, `>`, `&` and
// quotes as plain text (they are allowed in free text and in firm names). So a name like
// `<img src=x onerror=...>` can reach a picker row and a party card. React escapes text, and the
// app writes no raw HTML into the page; this test proves what the user would see.
//
// Pass means: the text is on the screen exactly as typed, no element was made from it, no
// attribute handler was made from it, and nothing ran.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { formNavigationProps, useEscapeDispatcher } from '../../src/shared/keyboard';
import * as searchApi from '../../src/shared/search/api';
import * as accountApi from '../../src/shared/accounts/api';
import { AccountPicker, Field, type PickerValue } from '../../src/shared/ui';
import * as urduApi from '../../src/shared/urdu/api';
import { account, fakeAccountSearch } from '../helpers/accounts';
import { message, renderIn, visible } from '../helpers/render';

vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const IMG = '<img src=x onerror="window.__pwned=1">';
const SCRIPT = '<script>window.__pwned=1</script>';
const SVG = '"><svg onload=window.__pwned=1>';
const ENTITY = '&lt;b&gt;bold&lt;/b&gt; &amp; <b>x</b>';
const LONG = 'x'.repeat(100);

const ROWS = [
  account(61, 'buyer', '03040061', IMG, IMG),
  account(62, 'buyer', '03040062', SCRIPT, SCRIPT),
  account(63, 'buyer', '03040063', ENTITY, SVG),
  account(64, 'buyer', '03040064', 'ن'.repeat(100), LONG),
];

declare global {
  interface Window {
    __pwned?: number;
  }
}

function Harness() {
  const [value, setValue] = useState<PickerValue | null>(null);
  useEscapeDispatcher();
  return (
    <form {...formNavigationProps}>
      <Field label={message('en', 'common-back')}>
        {(control) => (
          <AccountPicker
            {...control}
            scope="buyer"
            value={value}
            onChange={(hit) => {
              setValue(
                hit === null
                  ? null
                  : { id: hit.id, code: hit.code, name_ur: hit.name_ur, name_en: hit.name_en },
              );
            }}
          />
        )}
      </Field>
    </form>
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  // `delete`, not `= undefined`: the flag is optional and exactOptionalPropertyTypes is on.
  delete window.__pwned;
  fakeAccountSearch(ROWS);
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
  vi.mocked(searchApi.searchMasters).mockResolvedValue([]);
  vi.mocked(searchApi.searchRecent).mockResolvedValue([]);
  vi.mocked(accountApi.accountsSimilar).mockResolvedValue([]);
});

function nothingWasMade(root: HTMLElement) {
  // No element came from the text: the only img, svg and script elements are the app's own.
  expect(root.querySelector('script')).toBeNull();
  expect(root.querySelector('img[src="x"]')).toBeNull();
  expect(root.querySelector('svg[onload]')).toBeNull();
  expect(root.querySelector('b')).toBeNull();
  // No element has an inline handler.
  const handlers = Array.from(root.querySelectorAll('*')).filter((element) =>
    element.getAttributeNames().some((name) => name.startsWith('on')),
  );
  expect(handlers).toEqual([]);
  expect(window.__pwned).toBeUndefined();
}

describe.each(LANGUAGES)('a picker with hostile names (%s)', (language: Language) => {
  it('shows every name as the text it is, and runs nothing', async () => {
    act(() => {
      useLanguageStore.setState({ language });
    });
    const { container } = renderIn(
      language,
      <QueryClientProvider
        client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
      >
        <Harness />
      </QueryClientProvider>,
    );
    const input = screen.getByRole<HTMLInputElement>('combobox');
    const user = userEvent.setup();
    await user.click(input);
    // Alt+Down opens the full list without typing (PRD FR-7.16).
    await user.keyboard('{Alt>}{ArrowDown}{/Alt}');
    await waitFor(() => {
      expect(screen.queryAllByRole('option')).toHaveLength(ROWS.length);
    });
    const shown = screen.getAllByRole('option').map((o) => visible(o.textContent));
    const wanted =
      language === 'ur' ? [IMG, SCRIPT, ENTITY, 'ن'.repeat(100)] : [IMG, SCRIPT, SVG, LONG];
    for (const text of wanted) {
      expect(
        shown.some((row) => row.includes(text)),
        `the text ${text.slice(0, 30)} is not shown literally`,
      ).toBe(true);
    }
    nothingWasMade(container);
    nothingWasMade(document.body);
    // Taking one fills the field with its text, still as text.
    await user.keyboard('{ArrowDown}{Enter}');
    await waitFor(() => {
      expect(input.value).not.toBe('');
    });
    nothingWasMade(document.body);
  });
});
