// A one-time person (پختہ کھاتہ) in the dead amount in the person field (plan 40, PRD FR-10.10, FR-13.5):
// a field that takes dead people (the receipt screen) asks for them and lists them with the tag "ڈوبی"; any
// other field never asks, and a person without the mark has no tag. In the Urdu and in the English app.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import * as pukhtaApi from '../../src/features/pukhta_khata/api';
import { PukhtaPersonPicker } from '../../src/features/pukhta_khata';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { Field } from '../../src/shared/ui';
import { NASIR } from '../helpers/bikri-saved';
import { PEOPLE_INCLUDE_DEAD } from '../helpers/dead-real';
import { message, renderIn, visible } from '../helpers/render';

vi.mock('../../src/features/pukhta_khata/api');

function Harness({ includeDead }: { includeDead: boolean }) {
  const [text, setText] = useState('');
  return (
    <Field label={message('en', 'vouchers-field-person')}>
      {(control) => (
        <PukhtaPersonPicker
          {...control}
          side="buyer"
          text={text}
          onTextChange={setText}
          onPick={() => undefined}
          includeDead={includeDead}
        />
      )}
    </Field>
  );
}

describe.each(LANGUAGES)(
  'the one-time person field and the dead amount (%s)',
  (language: Language) => {
    beforeEach(() => {
      vi.resetAllMocks();
      act(() => {
        useLanguageStore.setState({ language });
      });
    });

    function open(includeDead: boolean) {
      renderIn(
        language,
        <QueryClientProvider
          client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
        >
          <Harness includeDead={includeDead} />
        </QueryClientProvider>,
      );
      return { user: userEvent.setup(), input: screen.getByRole<HTMLInputElement>('combobox') };
    }

    it('a field that takes dead people asks for them and tags the one who is dead', async () => {
      vi.mocked(pukhtaApi.searchPukhtaPeople).mockResolvedValue(PEOPLE_INCLUDE_DEAD);
      const { user, input } = open(true);
      await user.click(input);
      await user.keyboard('nasir');
      const option = await screen.findByRole('option');
      expect(visible(option.textContent)).toContain(message(language, 'dead-tag'));
      expect(pukhtaApi.searchPukhtaPeople).toHaveBeenCalledWith('nasir', 'buyer', true);
    });

    it('any other field asks the plain way, and a person who is not dead has no tag', async () => {
      vi.mocked(pukhtaApi.searchPukhtaPeople).mockResolvedValue([NASIR]);
      const { user, input } = open(false);
      await user.click(input);
      await user.keyboard('nasir');
      const option = await screen.findByRole('option');
      expect(visible(option.textContent)).not.toContain(message(language, 'dead-tag'));
      expect(pukhtaApi.searchPukhtaPeople).toHaveBeenCalledWith('nasir', 'buyer');
    });
  },
);
