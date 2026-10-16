// The sign-in screen: the form, or the recovery path with the recovery code (plan 04).

import { useState } from 'react';
import type { AuthStatus } from '../../../bindings/AuthStatus';
import { useEscape } from '../../../shared/keyboard';
import { AuthLayout } from '../components/AuthLayout';
import { LoginForm } from '../components/LoginForm';
import { RecoverForm } from '../components/RecoverForm';

interface LoginPageProps {
  status: AuthStatus;
}

export function LoginPage({ status }: LoginPageProps) {
  const [recovering, setRecovering] = useState(false);
  // Esc goes back from the recovery form to the sign-in form (PRD FR-7.18).
  useEscape(
    'leave',
    () => {
      setRecovering(false);
    },
    recovering,
  );
  return (
    <AuthLayout firm={status.firm} version={status.app_version}>
      {recovering ? (
        <RecoverForm
          onBack={() => {
            setRecovering(false);
          }}
        />
      ) : (
        <LoginForm
          onForgot={() => {
            setRecovering(true);
          }}
        />
      )}
    </AuthLayout>
  );
}
