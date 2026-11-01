// A screen or list could not load (design system 6, Code Standards F8). Plain words from
// errors.ftl, what to do, the reference number in Latin digits (NFR-REL-6, NFR-OBS-3), and a
// retry button. Never a raw error code (NFR-USE-14).

import { PartnerLockedBox } from './PartnerLockedBox';
import { Localized } from '@fluent/react';
import { CircleAlert } from 'lucide-react';
import type { ReactNode } from 'react';
import type { UiError } from '../../bindings/UiError';
import { Button } from './Button';
import { Code } from './Code';
import { ErrorMessage } from './ErrorMessage';
import { StateBadge } from './StateBadge';

interface ErrorStateProps {
  /** One line: what could not be done, for example "فہرست نہیں کھل سکی". */
  title: ReactNode;
  error: UiError;
  onRetry: () => void;
}

export function ErrorState({ title, error, onRetry }: ErrorStateProps) {
  // A report or a screen that the partner code keeps closed does not fail: it asks for the code
  // (plan 44, PRD FR-17.11). When the code is right every answer is read again and it opens.
  if (error.errors.some((item) => item.code === 'ERR_PARTNER_LOCKED')) {
    return (
      <div className="flex flex-col items-center justify-center gap-3 p-4 text-center">
        <PartnerLockedBox />
      </div>
    );
  }
  return (
    <div role="alert" className="flex flex-col items-center justify-center gap-3 p-4 text-center">
      <StateBadge icon={CircleAlert} tone="due" />
      <h2 className="type-section text-due">{title}</h2>
      {error.errors.map((item, index) => (
        <ErrorMessage key={`${item.code}-${String(index)}`} item={item}>
          <p className="type-body text-ink-muted" />
        </ErrorMessage>
      ))}
      {error.ref === '' ? null : (
        <Localized id="err-reference" vars={{ ref: error.ref }} elems={{ code: <Code /> }}>
          <p className="type-body text-ink-muted" />
        </Localized>
      )}
      <Button onClick={onRetry}>
        <Localized id="common-retry" />
      </Button>
    </div>
  );
}
