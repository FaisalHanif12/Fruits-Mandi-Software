// اردو | English, always visible in the top bar (PRD FR-7.30, design system 6). Alt+L also
// switches. Each language name is written in its own script and font.

import { Localized, useLocalization } from '@fluent/react';
import type { Language } from '../../i18n';
import { useLanguageStore } from '../../shared/i18n';
import { SHORTCUTS, useShortcut } from '../../shared/keyboard';
import { SegmentedControl, type SegmentedOption } from '../../shared/ui';

const OPTIONS: readonly SegmentedOption<Language>[] = [
  {
    value: 'ur',
    lang: 'ur',
    label: (
      <Localized id="common-language-ur">
        <span />
      </Localized>
    ),
  },
  {
    value: 'en',
    lang: 'en',
    label: (
      <Localized id="common-language-en">
        <span />
      </Localized>
    ),
  },
];

export function LanguageSwitch() {
  const { l10n } = useLocalization();
  const language = useLanguageStore((state) => state.language);
  const setLanguage = useLanguageStore((state) => state.setLanguage);
  const toggleLanguage = useLanguageStore((state) => state.toggleLanguage);
  useShortcut(SHORTCUTS.languageToggle, toggleLanguage);

  return (
    <SegmentedControl
      options={OPTIONS}
      value={language}
      onChange={setLanguage}
      groupLabel={l10n.getString('common-language-switch')}
    />
  );
}
