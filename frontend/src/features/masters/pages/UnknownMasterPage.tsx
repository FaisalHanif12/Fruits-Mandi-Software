// An address under /index that is not one of the ten masters: says so and offers the way back.

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { SearchX } from 'lucide-react';
import { Button, EmptyState } from '../../../shared/ui';

export function UnknownMasterPage() {
  const navigate = useNavigate();
  return (
    <EmptyState
      icon={SearchX}
      title={<Localized id="masters-unknown-title" />}
      help={<Localized id="masters-unknown-help" />}
      action={
        <Button
          variant="primary"
          onClick={() => {
            void navigate({ to: '/index' });
          }}
        >
          <Localized id="masters-menu-back" />
        </Button>
      }
    />
  );
}
