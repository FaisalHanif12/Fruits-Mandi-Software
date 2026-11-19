// A bar for a stage that can tell how far it is (the download of a copy). It is drawn only when there
// is a measure; a stage with none shows its words alone. The figure comes from the backend, 0 to 100.

import { useLocalization } from '@fluent/react';

interface ProgressBarProps {
  /** 0 to 100. */
  percent: number;
}

export function ProgressBar({ percent }: ProgressBarProps) {
  const { l10n } = useLocalization();
  return (
    <div
      role="progressbar"
      aria-label={l10n.getString('restore-progress-label')}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={percent}
      className="h-2 w-full overflow-hidden rounded-full bg-surface-sunken"
    >
      <div className="h-full bg-primary" style={{ inlineSize: `${String(percent)}%` }} />
    </div>
  );
}
