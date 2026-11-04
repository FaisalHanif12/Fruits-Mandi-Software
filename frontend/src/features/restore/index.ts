// What other modules may use from the restore module (Tech Stack 5.3): the first screen of a new
// install and the restore pages (shown by the app's first-run gate), the reminders after the first
// sign-in on a restored computer, and the choice the gate needs to read.
export { AfterRestoreCard } from './components/AfterRestoreCard';
export { RestorePage } from './pages/RestorePage';
export { StartPage } from './pages/StartPage';
export { useRestoreStore, type FirstRunChoice } from './store';
