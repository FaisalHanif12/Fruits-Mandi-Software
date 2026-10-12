// The add and edit screen of a rule (route /index/rules/new and /index/rules/<id>). It reads what
// the form needs (the rule for an edit, today for a new one) and shows the four states: loading,
// error with a retry, "not found", and the form (PRD FR-7.46, Code Standards F8).

import { Localized } from '@fluent/react';
import { useNavigate, useParams } from '@tanstack/react-router';
import { SearchX } from 'lucide-react';
import { useDayStatus } from '../../../app/use-day-status';
import { uiErrorOf } from '../../../shared/api';
import { Button, EmptyState, ErrorState, LoadingState } from '../../../shared/ui';
import { DefaultRuleView } from '../components/DefaultRuleView';
import { RuleForm } from '../components/RuleForm';
import { useRule } from '../hooks';

export function RuleFormPage() {
  const { id } = useParams({ strict: false });
  if (id === undefined) return <NewRule />;
  const number = Number(id);
  return Number.isInteger(number) ? <EditRule id={number} /> : <NotFound />;
}

function NewRule() {
  const day = useDayStatus();
  if (day.isPending) return <LoadingState />;
  return <RuleForm rule={null} today={day.data?.today ?? ''} />;
}

function EditRule({ id }: { id: number }) {
  const rule = useRule(id);
  if (rule.isPending) return <LoadingState />;
  if (rule.isError) {
    return (
      <ErrorState
        title={<Localized id="rules-load-failed" />}
        error={uiErrorOf(rule.error)}
        onRetry={() => {
          void rule.refetch();
        }}
      />
    );
  }
  // The three global defaults are changed only in Settings, "Charges".
  if (rule.data.is_default) return <DefaultRuleView rule={rule.data} />;
  return <RuleForm key={rule.data.id} rule={rule.data} today="" />;
}

function NotFound() {
  const navigate = useNavigate();
  return (
    <EmptyState
      icon={SearchX}
      title={<Localized id="rules-not-found-title" />}
      help={<Localized id="rules-not-found-help" />}
      action={
        <Button
          variant="primary"
          onClick={() => {
            void navigate({ to: '/index/rules' });
          }}
        >
          <Localized id="common-back" />
        </Button>
      }
    />
  );
}
