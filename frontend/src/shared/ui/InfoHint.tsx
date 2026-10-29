// A hint that is not always on screen (plan 38 G5, PRD FR-7.1): a small ⓘ button next to a label that
// opens the hint in a bubble, by click, Enter or Space, and closes with Esc. It replaces a grey
// paragraph under a field, so a form stays short. The words are given as a Fluent message by the
// screen, and a screen reader reads them from the button's description.

import { useLocalization } from '@fluent/react';
import { Info } from 'lucide-react';
import { Popover } from 'radix-ui';
import type { ReactNode } from 'react';
import { useUiLanguage } from '../i18n';

interface InfoHintProps {
  children: ReactNode;
}

export function InfoHint({ children }: InfoHintProps) {
  const { l10n } = useLocalization();
  const { language, direction } = useUiLanguage();
  return (
    <Popover.Root>
      <Popover.Trigger
        aria-label={l10n.getString('common-info-label')}
        className="-my-1 inline-flex size-target shrink-0 items-center justify-center rounded-1 text-ink-muted hover:text-ink"
      >
        <Info aria-hidden="true" className="size-icon-md" />
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Content
          lang={language}
          dir={direction}
          sideOffset={4}
          className="z-50 w-form max-w-full rounded-2 border border-border bg-surface p-3 type-body text-ink shadow-overlay"
        >
          {children}
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}
