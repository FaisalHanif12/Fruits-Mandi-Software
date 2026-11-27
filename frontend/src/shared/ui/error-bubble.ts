// How long the error bubble of a compact-row field stays on screen (plan 93 T8, Umar 22-11-2026), in milliseconds
// (Code Standards G8). It shows again each time the user repeats the action that gives the error. It is the one value of
// every short note of an entry screen (`NOTICE_MS`, plan 97 item 4: 5 seconds).
import { NOTICE_MS } from '../hooks';

export const ERROR_BUBBLE_MS = NOTICE_MS;
