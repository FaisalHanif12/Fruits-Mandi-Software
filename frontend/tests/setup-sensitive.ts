// The extra setup of the `sensitive` test project (vite.config.ts): the files that type into pickers
// one after another with the backend's real answers. On a busy machine an answer, a list or a pick
// takes longer than the one second Testing Library waits by default, so a wait that is only slow is
// not a failure here. A wait that never ends still fails, after five seconds.
import { configure } from '@testing-library/react';

const SLOW_MACHINE_WAIT_MS = 5_000;

configure({ asyncUtilTimeout: SLOW_MACHINE_WAIT_MS });
