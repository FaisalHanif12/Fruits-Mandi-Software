// Shows the toasts: nav-coloured ground, white text, an icon, at the bottom end corner
// (design system 6, UsersEn board). Each one disappears by itself and never needs an OK click
// (PRD FR-7.7). An error toast is announced at once; a success toast politely.
// Rendered once, in the app providers.

import { useLocalization } from '@fluent/react';
import { CircleAlert, CircleCheck } from 'lucide-react';
import { Toast } from 'radix-ui';
import { useUiLanguage } from '../i18n';
import { useToastStore, type ToastTone } from './toast-store';

/** How long a toast stays (PRD FR-7.7 says it disappears by itself). */
export const TOAST_DURATION_MS = 5_000;

const ICONS = {
  success: (
    <CircleCheck aria-hidden="true" className="size-icon-md shrink-0 text-money-in-on-nav" />
  ),
  error: <CircleAlert aria-hidden="true" className="size-icon-md shrink-0 text-due-on-nav" />,
} as const satisfies Record<ToastTone, unknown>;

export function Toaster() {
  const { l10n } = useLocalization();
  const { language, direction } = useUiLanguage();
  const toasts = useToastStore((state) => state.toasts);
  const dismiss = useToastStore((state) => state.dismiss);
  return (
    // Swipe down closes a toast: the same gesture in both directions. No hotkey: F8 is not one
    // of our keys (PRD 7.3).
    <Toast.Provider
      duration={TOAST_DURATION_MS}
      swipeDirection="down"
      label={l10n.getString('common-notifications')}
    >
      {toasts.map((toast) => (
        <Toast.Root
          key={toast.id}
          type={toast.tone === 'error' ? 'foreground' : 'background'}
          onOpenChange={(open) => {
            if (!open) dismiss(toast.id);
          }}
          lang={language}
          dir={direction}
          className="flex items-center gap-3 rounded-2 bg-nav px-4 py-3 text-on-primary shadow-overlay"
        >
          {ICONS[toast.tone]}
          <Toast.Description className="type-body">{toast.message}</Toast.Description>
        </Toast.Root>
      ))}
      <Toast.Viewport
        hotkey={[]}
        label={l10n.getString('common-notifications')}
        dir={direction}
        className="fixed end-6 bottom-7 z-50 flex max-w-toast flex-col gap-2 outline-none"
      />
    </Toast.Provider>
  );
}
