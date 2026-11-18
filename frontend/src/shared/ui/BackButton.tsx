// The Back button of every page that was opened from another page (plan 78 F1, PRD FR-7.18). It stands
// in one place, at the start of the top bar, so a page never has its own. Its arrow turns round in the
// right-to-left layout (Code Standards F26). From 1366 px wide it shows its word and its key (Esc); in a
// narrower window (the top bar is crowded beside the side menu, NFR-USE-15) it is the arrow alone, and the
// word is still its name for a screen reader (aria-label) and its tip.

import { Localized, useLocalization } from '@fluent/react';
import { ArrowLeft } from 'lucide-react';
import { SHORTCUTS } from '../keyboard';
import { Button } from './Button';
import { KeyChip } from './KeyChip';

interface BackButtonProps {
  onClick: () => void;
}

export function BackButton({ onClick }: BackButtonProps) {
  const { l10n } = useLocalization();
  return (
    <Button
      title={l10n.getString('common-back')}
      accessibleName={l10n.getString('common-back')}
      onClick={onClick}
    >
      <ArrowLeft aria-hidden="true" className="size-icon-md rtl:rotate-180" />
      <span className="hidden normal:inline">
        <Localized id="common-back" />
      </span>
      <span className="hidden normal:inline-flex">
        <KeyChip shortcut={SHORTCUTS.back} tone="on-light" />
      </span>
    </Button>
  );
}
