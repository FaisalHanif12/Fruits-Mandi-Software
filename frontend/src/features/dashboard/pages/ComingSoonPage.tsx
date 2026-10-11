// A part of the app that is not built yet (PRD FR-7.46): says so plainly and offers the way back.
// Used for every menu item whose module comes in a later plan.

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { Hammer } from 'lucide-react';
import { Button, EmptyState } from '../../../shared/ui';

export function ComingSoonPage() {
  const navigate = useNavigate();
  return (
    <div className="flex h-full items-center justify-center">
      <EmptyState
        icon={Hammer}
        title={<Localized id="soon-title" />}
        help={<Localized id="soon-help" />}
        action={
          <Button
            variant="primary"
            onClick={() => {
              void navigate({ to: '/' });
            }}
          >
            <Localized id="soon-back" />
          </Button>
        }
      />
    </div>
  );
}
