// Development page /dev/login (plan 75): the real sign-in screen with its brand panel, with sample
// firm names and no backend, so the panel can be opened in a plain browser tab and photographed in
// both languages and at every window size. Add `#recover` to the address for the recovery form.
// Never shipped: the route exists only when import.meta.env.DEV is true (app/router.tsx).

import { LoginPage } from '../../features/auth/pages/LoginPage';
import type { AuthStatus } from '../../bindings/AuthStatus';

const SAMPLE_STATUS: AuthStatus = {
  first_run: false,
  session: null,
  locked: false,
  login_language: 'ur',
  firm: {
    name_ur: 'مہر عدنان اینڈ برادرز',
    name_en: 'Maher Adnan and Brothers',
    mandi_ur: 'نئی سبزی منڈی، کوٹ رادھا کشن',
    mandi_en: 'New Sabzi Mandi, Kot Radha Kishan',
  },
  app_version: '0.1.12',
};

export function LoginDevPage() {
  return <LoginPage status={SAMPLE_STATUS} />;
}
