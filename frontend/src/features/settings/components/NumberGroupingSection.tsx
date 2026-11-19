// Number grouping (PRD Section 21, Language; FR-19.7): 1,316,040 or 13,16,040. It is one value for
// the whole app: a press changes every amount on screen at once (the grouping store) and is kept for
// the next start. The two examples are numbers, so they are set apart with <Num> and never flip in
// the Urdu text.

import { Localized, useLocalization } from '@fluent/react';
import type { Grouping } from '../../../bindings/Grouping';
import { uiErrorOf } from '../../../shared/api';
import {
  ErrorNotice,
  Num,
  SegmentedControl,
  showToast,
  type SegmentedOption,
} from '../../../shared/ui';
import { useSetNumberGrouping } from '../maintenance-hooks';

const OPTIONS: readonly SegmentedOption<Grouping>[] = [
  {
    value: 'international',
    label: (
      <Localized id="settings-grouping-international" elems={{ num: <Num /> }}>
        <span />
      </Localized>
    ),
  },
  {
    value: 'lakh',
    label: (
      <Localized id="settings-grouping-lakh" elems={{ num: <Num /> }}>
        <span />
      </Localized>
    ),
  },
];

export function NumberGroupingSection({ grouping }: { grouping: Grouping }) {
  const { l10n } = useLocalization();
  const set = useSetNumberGrouping();
  const error = set.error === null ? null : uiErrorOf(set.error);
  return (
    <section className="flex flex-col gap-3">
      <Localized id="settings-grouping-title">
        <h2 className="type-section" />
      </Localized>
      <Localized id="settings-grouping-help">
        <p className="type-body text-ink-muted" />
      </Localized>
      {error === null ? null : <ErrorNotice error={error} />}
      <div>
        <SegmentedControl
          options={OPTIONS}
          value={grouping}
          groupLabel={l10n.getString('settings-grouping-title')}
          onChange={(next) => {
            set.mutate(next, {
              onSuccess: () => {
                showToast('success', l10n.getString('settings-saved'));
              },
            });
          }}
        />
      </div>
    </section>
  );
}
