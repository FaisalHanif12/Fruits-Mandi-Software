// Language and text size (PRD FR-7.11, FR-7.30, plan 04). The default language is for NEW users and
// is the Owner's setting; every user changes his own language from the top bar and his own text
// size here or with Ctrl +, Ctrl − and Ctrl 0. The size stops where the content would be narrower
// than 1024 points (NFR-USE-18); the buttons stop there too.

import { Localized, useLocalization } from '@fluent/react';
import type { Grouping } from '../../../bindings/Grouping';
import type { Language } from '../../../bindings/Language';
import {
  DEFAULT_TEXT_SIZE,
  effectiveTextSize,
  stepTextSize,
  useTextSizeStore,
} from '../../../app/layout/text-size';
import { uiErrorOf } from '../../../shared/api';
import { SHORTCUTS } from '../../../shared/keyboard';
import {
  Button,
  ErrorNotice,
  Num,
  SegmentedControl,
  showToast,
  type SegmentedOption,
} from '../../../shared/ui';
import { useSetDefaultLanguage } from '../hooks';
import { LearnedWords } from './LearnedWords';
import { NumberGroupingSection } from './NumberGroupingSection';

const LANGUAGE_OPTIONS: readonly SegmentedOption<Language>[] = [
  { value: 'ur', lang: 'ur', label: <Localized id="common-language-ur" /> },
  { value: 'en', lang: 'en', label: <Localized id="common-language-en" /> },
];

interface LanguageTabProps {
  defaultLanguage: Language;
  numberGrouping: Grouping;
}

export function LanguageTab({ defaultLanguage, numberGrouping }: LanguageTabProps) {
  const { l10n } = useLocalization();
  const chosen = useTextSizeStore((state) => state.chosen);
  const setChosen = useTextSizeStore((state) => state.setChosen);
  const setDefault = useSetDefaultLanguage();
  const error = setDefault.error === null ? null : uiErrorOf(setDefault.error);
  const size = effectiveTextSize(chosen, window.innerWidth);
  return (
    <div className="flex max-w-reading flex-col gap-6">
      <section className="flex flex-col gap-3">
        <Localized id="settings-language-default">
          <h2 className="type-section" />
        </Localized>
        <Localized id="settings-language-default-help">
          <p className="type-body text-ink-muted" />
        </Localized>
        {error === null ? null : <ErrorNotice error={error} />}
        <div>
          <SegmentedControl
            options={LANGUAGE_OPTIONS}
            value={defaultLanguage}
            groupLabel={l10n.getString('settings-language-default')}
            onChange={(language) => {
              setDefault.mutate(
                { language },
                {
                  onSuccess: () => {
                    showToast('success', l10n.getString('settings-saved'));
                  },
                },
              );
            }}
          />
        </div>
      </section>
      <section className="flex flex-col gap-3">
        <Localized id="settings-text-size">
          <h2 className="type-section" />
        </Localized>
        <Localized id="settings-text-size-help">
          <p className="type-body text-ink-muted" />
        </Localized>
        <Localized
          id="settings-text-size-value"
          vars={{ size: String(size) }}
          elems={{ num: <Num /> }}
        >
          <p className="type-body" />
        </Localized>
        <div className="flex flex-wrap gap-2">
          <Button
            shortcut={SHORTCUTS.textSmaller}
            onClick={() => {
              setChosen(stepTextSize(size, -1, window.innerWidth));
            }}
          >
            <Localized id="settings-text-smaller" />
          </Button>
          <Button
            shortcut={SHORTCUTS.textBigger}
            onClick={() => {
              setChosen(stepTextSize(size, 1, window.innerWidth));
            }}
          >
            <Localized id="settings-text-bigger" />
          </Button>
          <Button
            shortcut={SHORTCUTS.textNormal}
            onClick={() => {
              setChosen(DEFAULT_TEXT_SIZE);
            }}
          >
            <Localized id="settings-text-normal" />
          </Button>
        </div>
      </section>
      <NumberGroupingSection grouping={numberGrouping} />
      <LearnedWords />
    </div>
  );
}
