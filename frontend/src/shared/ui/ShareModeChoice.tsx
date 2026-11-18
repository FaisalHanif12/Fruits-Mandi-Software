// The choice at the top of the shares of a new trip or garden (PRD FR-12.29, plan 78 D1, Umar 17-11-2026):
// "عام حصے" (default shares: the firm's, or the last trip's) or "اپنے حصے" (custom shares). Default shows the
// shares and cannot change them; custom opens every share for typing, up to 2 decimals, a partner who is not in it
// at 0, the total exactly 100 (the backend checks it, `partners_check_shares`). A line under the choice says what
// the chosen one means. The shares themselves are text on the form's own state (Code Standards F39); this only
// asks which of the two the Owner wants.

import { Localized, useLocalization } from '@fluent/react';
import type { ReactNode } from 'react';
import { SegmentedControl } from './SegmentedControl';

export type ShareMode = 'default' | 'custom';

interface ShareModeChoiceProps {
  mode: ShareMode;
  onChange: (mode: ShareMode) => void;
  /** What the default shares are ("the firm's", "the last trip's"), shown while the default is chosen: a Fluent message. */
  defaultHelp: ReactNode;
}

export function ShareModeChoice({ mode, onChange, defaultHelp }: ShareModeChoiceProps) {
  const { l10n } = useLocalization();
  return (
    <div className="flex flex-col gap-2">
      <SegmentedControl<ShareMode>
        groupLabel={l10n.getString('ventures-share-mode-label')}
        value={mode}
        onChange={onChange}
        options={[
          { value: 'default', label: <Localized id="ventures-share-mode-default" /> },
          { value: 'custom', label: <Localized id="ventures-share-mode-custom" /> },
        ]}
      />
      {mode === 'default' ? (
        defaultHelp
      ) : (
        <Localized id="ventures-share-mode-custom-help">
          <p className="type-body text-ink-muted" />
        </Localized>
      )}
    </div>
  );
}
