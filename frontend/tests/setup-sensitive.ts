// The extra setup of the `sensitive` test project (vite.config.ts): the files that type into pickers
// one after another with the backend's real answers. On a busy machine an answer, a list or a pick
// takes longer than the one second Testing Library waits by default, so a wait that is only slow is
// not a failure here. A wait that never ends still fails, after thirty seconds.
//
// Thirty, not five: the first move to a screen in a fresh test worker loads that screen's code (a lazy
// route) and Vite transforms its whole tree first. For the sale book that took more than five seconds
// on a busy machine even at a load of 45, so "Enter on a row opens the بکری" (bikri-past-return.test.tsx)
// failed in Urdu and in English, and passed with a longer wait. A slow CI runner meets the same thing.
import { configure } from '@testing-library/react';

const SLOW_MACHINE_WAIT_MS = 30_000;

configure({ asyncUtilTimeout: SLOW_MACHINE_WAIT_MS });
