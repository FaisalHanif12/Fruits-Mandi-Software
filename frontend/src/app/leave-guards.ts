// The screens that guard their unsaved work (LeaveGuard) can be asked together when the user leaves
// the whole app by logging out (PRD FR-5.6, FR-7.9, plan 75 T3). Each mounted LeaveGuard lists itself
// here. Before a log out, `confirmLeaveAll` asks every guard that has unsaved work, one after the other,
// with the same "Save changes?" box it uses for leaving its screen. Any "back" stops the log out. The
// idle lock (PRD FR-8.4) never asks, so it does not come through here.

export interface LeaveGuardEntry {
  /** True while the screen has changes that are not saved. */
  isDirty: () => boolean;
  /** Shows the guard's box. Resolves true to go on (saved, or left without saving), false to stay. */
  ask: () => Promise<boolean>;
}

const guards = new Set<LeaveGuardEntry>();

/** Lists a guard. Returns the function that takes it off the list again. */
export function registerLeaveGuard(entry: LeaveGuardEntry): () => void {
  guards.add(entry);
  return () => {
    guards.delete(entry);
  };
}

/** True when there is no unsaved work, or the user chose to save it or to leave it. */
export async function confirmLeaveAll(): Promise<boolean> {
  for (const guard of [...guards]) {
    if (guard.isDirty() && !(await guard.ask())) return false;
  }
  return true;
}
