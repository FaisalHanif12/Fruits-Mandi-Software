// What other modules may use from the backup module (Tech Stack 5.3): the status bar item and its
// words, the state of the backups, the window-close overlay, and the key card of the first-run
// wizard's key step.
export { BackupExitOverlay } from './components/BackupExitOverlay';
export { BackupKeyCard } from './components/BackupKeyCard';
export { BackupStatusItem, statusBarWords } from './components/BackupStatusItem';
export { useBackupStatus } from './hooks';
export { useBackupKeyStore } from './key-store';
