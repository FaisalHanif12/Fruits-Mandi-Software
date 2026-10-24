// The public side of the `dashboard` module (Tech Stack 5.2): what other modules may import. The
// attention bell in the top bar and the guide are in `app/`; they use these.
export { AttentionLines } from './components/AttentionLines';
export { useOpenDestination, type Destination } from './destinations';
export { ATTENTION_KEY, DASHBOARD_KEY, useAttention, useDashboard } from './hooks';
