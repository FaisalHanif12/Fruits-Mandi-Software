// The box that stands where partner figures would be, while the code is not entered (plan 44, PRD
// FR-17.11): "Enter the code". The rest of the screen works. The button opens the code box (made once
// in the shell); when the code is right every figure is read again and shows.

import { Localized } from '@fluent/react';
import { LockKeyhole } from 'lucide-react';
import { Button } from './Button';
import { Notice } from './Notice';
import { openPartnerCode } from './partner-code-store';

export function PartnerLockedBox() {
  return (
    <Notice
      tone="info"
      action={
        <Button onClick={openPartnerCode}>
          <LockKeyhole aria-hidden="true" className="size-icon-md" />
          <Localized id="partners-code-enter" />
        </Button>
      }
    >
      <Localized id="partners-code-box-text" />
    </Notice>
  );
}
