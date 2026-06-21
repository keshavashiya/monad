/**
 * Session state — persists lightweight data across page reloads via sessionStorage.
 *
 * Known hosts key acceptance is stored so the host key prompt only appears once
 * per browser tab session. Nothing sensitive is stored. No tracking.
 */

const STORAGE_KEY = 'monad:state';

interface MonadState {
  knownHosts: boolean;
  visited: number;
}

function load(): MonadState {
  try {
    const raw = sessionStorage.getItem(STORAGE_KEY);
    if (raw) return JSON.parse(raw);
  } catch { /* ignore */ }
  return { knownHosts: false, visited: 0 };
}

function save(state: MonadState): void {
  try {
    sessionStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch { /* ignore */ }
}

export const state: MonadState = load();

/** Mark that the user has accepted the host key this session */
export function acceptHostKey(): void {
  state.knownHosts = true;
  state.visited += 1;
  save(state);
}

/** Check if the host key has been accepted this session */
export function isHostKeyAccepted(): boolean {
  return state.knownHosts;
}

/** Initial visited flag for boot sequence */
export function isFirstVisit(): boolean {
  return state.visited === 0;
}
