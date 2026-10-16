// The add and edit screen of an account (route /index/accounts/new and /index/accounts/<id>). It
// reads what the form needs (the account for an edit, today for a new one) and shows the four
// states: loading, error with a retry, "not found", and the form (PRD FR-7.46, F8).

import { Localized } from '@fluent/react';
import { useNavigate, useParams } from '@tanstack/react-router';
import { SearchX } from 'lucide-react';
import { useDayStatus } from '../../../app/use-day-status';
import { uiErrorOf } from '../../../shared/api';
import { Button, EmptyState, ErrorState, LoadingState } from '../../../shared/ui';
import { useAccount } from '../../../shared/accounts';
import { AccountForm } from '../components/AccountForm';

export function AccountFormPage() {
  const { id } = useParams({ strict: false });
  if (id === undefined) return <NewAccount />;
  const number = Number(id);
  return Number.isInteger(number) ? <EditAccount id={number} /> : <NotFound />;
}

function NewAccount() {
  const day = useDayStatus();
  if (day.isPending) return <LoadingState />;
  if (day.isError) {
    return (
      <ErrorState
        title={<Localized id="accounts-load-failed" />}
        error={uiErrorOf(day.error)}
        onRetry={() => {
          void day.refetch();
        }}
      />
    );
  }
  return <AccountForm account={null} today={day.data.today} />;
}

function EditAccount({ id }: { id: number }) {
  const account = useAccount(id);
  if (account.isPending) return <LoadingState />;
  if (account.isError) {
    return (
      <ErrorState
        title={<Localized id="accounts-load-failed" />}
        error={uiErrorOf(account.error)}
        onRetry={() => {
          void account.refetch();
        }}
      />
    );
  }
  return <AccountForm key={account.data.id} account={account.data} today="" />;
}

function NotFound() {
  const navigate = useNavigate();
  return (
    <EmptyState
      icon={SearchX}
      title={<Localized id="accounts-not-found-title" />}
      help={<Localized id="accounts-not-found-help" />}
      action={
        <Button
          variant="primary"
          onClick={() => {
            void navigate({ to: '/index/accounts' });
          }}
        >
          <Localized id="common-back" />
        </Button>
      }
    />
  );
}
