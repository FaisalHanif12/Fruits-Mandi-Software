// The five steps of the first-run guide (PRD FR-7.12): the dashboard, the انڈکس, the بکری بک, the
// وصولی and the reports. Each step opens its screen when the guide reaches it, so the Owner sees the
// screen the words are about. The words are Fluent (`guide-step-<id>-title` and `-body`).

export type GuideStepId = 'dashboard' | 'index' | 'bikri' | 'receipt' | 'reports';

export interface GuideStep {
  id: GuideStepId;
  /** The address of the screen the step is about. */
  to: '/' | '/index' | '/bikri/book' | '/bikri/voucher' | '/reports';
  /** The voucher type of the screen, for the one step that needs it. */
  type?: 'receipt';
}

export const GUIDE_STEPS: readonly GuideStep[] = [
  { id: 'dashboard', to: '/' },
  { id: 'index', to: '/index' },
  { id: 'bikri', to: '/bikri/book' },
  { id: 'receipt', to: '/bikri/voucher', type: 'receipt' },
  { id: 'reports', to: '/reports' },
];

/** The name under which the guide's "seen" mark is kept (`prefs_screen_*`). */
export const GUIDE_SCREEN = 'app.guide';
