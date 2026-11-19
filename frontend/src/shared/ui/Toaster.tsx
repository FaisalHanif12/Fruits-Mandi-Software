// Shows the toasts: nav-coloured ground, white text, an icon, at the bottom end corner
// (design system 6, UsersEn board). Each one disappears by itself and never needs an OK click
// (PRD FR-7.7). An error toast is announced at once; a success toast politely.
// Rendered once, in the app providers.

import { useLocalization } from '@fluent/react';
import { CircleAlert, CircleCheck } from 'lucide-react';
import { Toast } from 'radix-ui';
import { useUiLanguage } from '../i18n';
import { useShortcut } from '../keyboard';
import { KeyChip } from './KeyChip';
import { useToastStore, type ToastAction, type ToastTone } from './toast-store';

/** How long a toast stays (PRD FR-7.7 says it disappears by itself). */
export const TOAST_DURATION_MS = 5_000;

const ICONS = {
  success: (
    <CircleCheck aria-hidden="true" className="size-icon-md shrink-0 text-money-in-on-nav" />
  ),
  error: <CircleAlert aria-hidden="true" className="size-icon-md shrink-0 text-due-on-nav" />,
} as const satisfies Record<ToastTone, unknown>;

/**
 * The button of a toast (PRD FR-7.47). Its key works only while the toast is on screen, because
 * this component is drawn with the toast; pressing the key or the button runs the action once and
 * closes the toast.
 */
function ToastActionButton({ action, onDone }: { action: ToastAction; onDone: () => void }) {
  const run = () => {
    action.onAction();
    onDone();
  };
  useShortcut(action.shortcut ?? NO_KEY, run, { enabled: action.shortcut !== undefined });
  return (
    <Toast.Action altText={action.altText} asChild>
      <button
        type="button"
        onClick={run}
        className="inline-flex h-control-compact shrink-0 items-center gap-2 rounded-2 border border-on-primary px-3 type-control text-on-primary"
      >
        {action.label}
        {action.shortcut === undefined ? null : (
          <KeyChip shortcut={action.shortcut} tone="on-primary" />
        )}
      </button>
    </Toast.Action>
  );
}

/** A key that is never pressed, for a toast whose button has no key (the hook is always called). */
const NO_KEY = { code: 'Unidentified', label: '', scope: 'anywhere' } as const;

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
          duration={toast.durationMs ?? TOAST_DURATION_MS}
          onOpenChange={(open) => {
            if (!open) dismiss(toast.id);
          }}
          lang={language}
          dir={direction}
          className="flex items-center gap-3 rounded-2 bg-nav px-4 py-3 text-on-primary shadow-overlay"
        >
          {ICONS[toast.tone]}
          <Toast.Description className="type-body">{toast.message}</Toast.Description>
          {toast.action === undefined ? null : (
            <ToastActionButton
              action={toast.action}
              onDone={() => {
                dismiss(toast.id);
              }}
            />
          )}
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
