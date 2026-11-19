// The checks of one copy, each with a tick or a cross and a word (NFR-BAK-5). Colour is never the
// only signal (NFR-USE-12): the words "passed" and "failed" go with the icon.

import { Localized, useLocalization } from '@fluent/react';
import { CircleCheck, CircleX } from 'lucide-react';
import type { FirstRunCheck } from '../types';

interface CheckListProps {
  checks: readonly FirstRunCheck[];
}

export function CheckList({ checks }: CheckListProps) {
  const { l10n } = useLocalization();
  return (
    <ul aria-label={l10n.getString('restore-check-list-label')} className="flex flex-col gap-1">
      {checks.map((check) => (
        <li key={check.name} className="flex items-center gap-2 type-body">
          {check.ok ? (
            <CircleCheck aria-hidden="true" className="size-icon-md shrink-0 text-money-in" />
          ) : (
            <CircleX aria-hidden="true" className="size-icon-md shrink-0 text-due" />
          )}
          <Localized id={`backup-check-${check.name}`} />
          <Localized id={check.ok ? 'backup-check-passed' : 'backup-check-failed'}>
            <span className={check.ok ? 'text-money-in' : 'text-due'} />
          </Localized>
        </li>
      ))}
    </ul>
  );
}
