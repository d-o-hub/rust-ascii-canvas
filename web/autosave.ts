/**
 * ASCII Canvas Editor — the autosave scheduler instance.
 *
 * `createAutoSaveScheduler` in `persistence.ts` is a factory; this module is the
 * single instance the app uses. It exists so that `eventResult.ts`, `events.ts`
 * and `events-file.ts` can all reach `scheduleAutoSave` / `flushAutoSave`
 * without importing `events.ts` (which imports them) for the sake of one object.
 *
 * The assignment onto `state` is the same load-time side effect `events.ts`
 * performed; it stays here so the owner of the scheduler is also the thing that
 * publishes it. Readers in `ui.ts` guard with `if (state.scheduleAutoSave)` and
 * run at event time, never at module-evaluation time.
 */

import { createAutoSaveScheduler } from './persistence.js';
import { state } from './state.js';

export const { schedule: scheduleAutoSave, flush: flushAutoSave } =
    createAutoSaveScheduler(() => state.editor);

state.scheduleAutoSave = scheduleAutoSave;
