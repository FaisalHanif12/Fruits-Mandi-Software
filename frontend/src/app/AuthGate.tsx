// Decides what the user sees (plan 04): the first-run wizard when there is no user yet, the
// sign-in screen when nobody is signed in, and otherwise the app. When the idle lock is on, the
// app stays on screen under the lock, so nothing typed is lost (PRD FR-8.4). Language and text
// size of the signed-in user are applied at sign-in and saved when he changes them.

import { Localized } from '@fluent/react';
import { Outlet, useNavigate, useRouterState } from '@tanstack/react-router';
import { useCallback, useEffect, useRef } from 'react';
import type { AuthStatus } from '../bindings/AuthStatus';
import type { SessionUser } from '../bindings/SessionUser';
import { LockOverlay } from '../features/auth/components/LockOverlay';
import { BikriDraftOffer } from '../features/bikri/draft-offer';
import { useAuthStatus, useLock, useLogout, useSavePreferences } from '../features/auth/hooks';
import { LoginPage } from '../features/auth/pages/LoginPage';
import { WizardPage } from '../features/auth/pages/WizardPage';
import { useSettings } from '../features/settings/hooks';
import { uiErrorOf } from '../shared/api';
import { isoToDisplayDate } from '../shared/format';
import { useLanguageStore } from '../shared/i18n';
import { SHORTCUTS, useEscape, useShortcut } from '../shared/keyboard';
import { ErrorState, LoadingState } from '../shared/ui';
import { AppShell } from './layout/AppShell';
import { MainNav } from './layout/MainNav';
import { ModuleMenu } from './layout/ModuleMenu';
import { MAIN_ROUTES, MODULE_ROUTES, mainNavOf, moduleMenuOf } from './nav-routes';
import { useDayStatus } from './use-day-status';
import { useIdleLock } from './use-idle-lock';

/** Before anyone has signed in, the screen is in the language last used on this computer. */
function useLoginLanguage(status: AuthStatus | undefined) {
  const applied = useRef(false);
  useEffect(() => {
    if (status === undefined || applied.current || status.session !== null) return;
    applied.current = true;
    useLanguageStore.getState().setLanguage(status.login_language);
  }, [status]);
}

function SignedIn({ user, locked }: { user: SessionUser; locked: boolean }) {
  const navigate = useNavigate();
  const pathname = useRouterState({ select: (state) => state.location.pathname });
  const settings = useSettings();
  const day = useDayStatus();
  const lock = useLock();
  const logout = useLogout();
  useSavePreferences(user);

  const { mutate: lockApp } = lock;
  const lockNow = useCallback(() => {
    lockApp();
  }, [lockApp]);
  useIdleLock(settings.data?.idle_lock_minutes, !locked, lockNow);

  const goTo = (to: string) => {
    void navigate({ to });
  };
  // A key from anywhere (PRD 7.3, NFR-USE-9).
  useShortcut(SHORTCUTS.newBikri, () => {
    goTo(MODULE_ROUTES['bikri-book']);
  });
  useShortcut(SHORTCUTS.newReceipt, () => {
    goTo(MODULE_ROUTES.receipt);
  });
  useShortcut(SHORTCUTS.newPayment, () => {
    goTo(MODULE_ROUTES.payment);
  });
  useShortcut(SHORTCUTS.newAdvance, () => {
    goTo(MODULE_ROUTES.advance);
  });
  useShortcut(SHORTCUTS.newExpense, () => {
    goTo(MAIN_ROUTES.rokar);
  });
  // Esc, when nothing else wants it, leaves the screen for the dashboard (PRD FR-7.18).
  useEscape(
    'leave',
    () => {
      goTo(MAIN_ROUTES.dashboard);
    },
    pathname !== '/' && !locked,
  );

  const main = mainNavOf(pathname);
  return (
    <>
      <AppShell
        nav={
          <MainNav
            current={main}
            onSelect={(id) => {
              goTo(MAIN_ROUTES[id]);
            }}
          />
        }
        moduleMenu={
          main === 'bikri' ? (
            <ModuleMenu
              current={moduleMenuOf(pathname)}
              onSelect={(id) => {
                goTo(MODULE_ROUTES[id]);
              }}
            />
          ) : null
        }
        userName={user.display_name}
        onLock={lockNow}
        onOpenAccount={(id) => {
          void navigate({ to: '/index/accounts/$id', params: { id: String(id) } });
        }}
        onOpenBikri={(id) => {
          void navigate({ to: '/bikri/book', search: { id } });
        }}
        onSignOut={() => {
          logout.mutate();
        }}
        date={day.data === undefined ? '' : isoToDisplayDate(day.data.today)}
        day={day.data === undefined ? null : day.data.today_open ? 'open' : 'closed'}
        closedUpTo={day.data?.closed_up_to == null ? null : isoToDisplayDate(day.data.closed_up_to)}
        onOpenDay={() => {
          goTo('/day');
        }}
      >
        <Outlet />
      </AppShell>
      {/* A بکری that was being typed when the app closed is offered back once (PRD FR-12.10). */}
      {locked ? null : <BikriDraftOffer />}
      {locked ? <LockOverlay userName={user.display_name} /> : null}
    </>
  );
}

export function AuthGate() {
  const status = useAuthStatus();
  useLoginLanguage(status.data);
  if (status.isPending) {
    return (
      <div className="min-h-screen bg-bg p-6">
        <LoadingState />
      </div>
    );
  }
  if (status.isError) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-bg p-6">
        <ErrorState
          title={<Localized id="layout-loading-failed" />}
          error={uiErrorOf(status.error)}
          onRetry={() => {
            void status.refetch();
          }}
        />
      </div>
    );
  }
  const current = status.data;
  if (current.first_run) return <WizardPage />;
  if (current.session === null) return <LoginPage status={current} />;
  return <SignedIn user={current.session} locked={current.locked} />;
}
