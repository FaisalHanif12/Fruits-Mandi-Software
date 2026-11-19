// What a computer with no data file shows (PRD FR-20.11, NFR-INS-2): the first screen with its two
// choices, then the first-run wizard of a new shop, or the restore pages that bring back the books of
// a lost computer. The choice lives in the restore store, so a language switch loses nothing. The
// wizard keeps its own store, so going back to the first screen and in again keeps what was typed.

import { WizardPage } from '../features/auth/pages/WizardPage';
import { RestorePage, StartPage, useRestoreStore } from '../features/restore';

export function FirstRunGate() {
  const choice = useRestoreStore((state) => state.choice);
  const setChoice = useRestoreStore((state) => state.setChoice);
  if (choice === 'new') {
    return (
      <WizardPage
        onBack={() => {
          setChoice('start');
        }}
      />
    );
  }
  if (choice === 'restore') return <RestorePage />;
  return <StartPage />;
}
