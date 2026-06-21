/**
 * Client-side memory — "the kernel forgets; your machine remembers."
 *
 * The MONAD kernel is stateless: it forgets everything when the session ends.
 * This module is the one place anything is remembered — and it lives *only* in
 * the visitor's own browser (IndexedDB, with a localStorage fallback). Nothing
 * is ever sent to a server. There is no server.
 *
 * It is the same local-first thesis behind `brain` and `docify`, applied to the
 * act of reading a résumé: your data never leaves your hardware.
 *
 * A synchronous in-memory `cache` is loaded once at boot so the `memory` command
 * can render instantly; writes update the cache and persist asynchronously.
 */

export interface MonadMemory {
  firstSeen: number; // epoch ms, 0 = never
  lastSeen: number; // epoch ms of the previous visit
  visits: number;
  commandCount: number;
  recentCommands: string[];
  hostKeyAccepted: boolean;
}

const DB_NAME = 'monad';
const STORE = 'kv';
const KEY = 'memory';
const LS_KEY = 'monad:memory';
const RECENT_MAX = 12;

// ANSI palette (matches the amber console theme).
const A = '\x1b[38;5;214m';
const B = '\x1b[1m';
const D = '\x1b[2m';
const G = '\x1b[38;5;113m';
const R = '\x1b[0m';

function defaults(): MonadMemory {
  return {
    firstSeen: 0,
    lastSeen: 0,
    visits: 0,
    commandCount: 0,
    recentCommands: [],
    hostKeyAccepted: false,
  };
}

let cache: MonadMemory = defaults();
let backend: 'IndexedDB' | 'localStorage' | 'memory-only' = 'memory-only';

function hasIDB(): boolean {
  try {
    return typeof indexedDB !== 'undefined';
  } catch {
    return false;
  }
}

function openDB(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, 1);
    req.onupgradeneeded = () => req.result.createObjectStore(STORE);
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

async function readStore(): Promise<Partial<MonadMemory> | null> {
  if (hasIDB()) {
    try {
      const db = await openDB();
      const val = await new Promise<Partial<MonadMemory> | null>((resolve, reject) => {
        const tx = db.transaction(STORE, 'readonly');
        const r = tx.objectStore(STORE).get(KEY);
        r.onsuccess = () => resolve((r.result as Partial<MonadMemory>) ?? null);
        r.onerror = () => reject(r.error);
      });
      backend = 'IndexedDB';
      return val;
    } catch {
      /* fall through to localStorage */
    }
  }
  try {
    const raw = localStorage.getItem(LS_KEY);
    backend = 'localStorage';
    return raw ? (JSON.parse(raw) as Partial<MonadMemory>) : null;
  } catch {
    backend = 'memory-only';
    return null;
  }
}

function persist(): void {
  // Fire-and-forget; never block the UI on storage.
  const snapshot = { ...cache };
  void (async () => {
    if (backend === 'IndexedDB') {
      try {
        const db = await openDB();
        await new Promise<void>((resolve, reject) => {
          const tx = db.transaction(STORE, 'readwrite');
          tx.objectStore(STORE).put(snapshot, KEY);
          tx.oncomplete = () => resolve();
          tx.onerror = () => reject(tx.error);
        });
        return;
      } catch {
        /* fall through */
      }
    }
    try {
      localStorage.setItem(LS_KEY, JSON.stringify(snapshot));
    } catch {
      /* memory-only: nothing persists, that's fine */
    }
  })();
}

/** Load persisted memory into the synchronous cache. Call once before boot. */
export async function initMemory(): Promise<MonadMemory> {
  const stored = await readStore();
  if (stored) cache = { ...defaults(), ...stored };
  return cache;
}

/** The current memory (synchronous; reflects the cache). */
export function getMemory(): MonadMemory {
  return cache;
}

/** True if this visitor has been here before. Read *before* `beginVisit`. */
export function isReturningVisitor(): boolean {
  return cache.visits > 0;
}

/** Record that a session has started. Call once, after boot completes. */
export function beginVisit(): void {
  const now = Date.now();
  if (cache.firstSeen === 0) cache.firstSeen = now;
  cache.visits += 1;
  cache.lastSeen = now;
  persist();
}

/** Persistently remember that the host key was accepted (skips the prompt next time). */
export function rememberHostKey(): void {
  cache.hostKeyAccepted = true;
  persist();
}

/** Append a command to the rolling history. */
export function recordCommand(cmd: string): void {
  if (!cmd) return;
  cache.commandCount += 1;
  cache.recentCommands.push(cmd);
  if (cache.recentCommands.length > RECENT_MAX) {
    cache.recentCommands = cache.recentCommands.slice(-RECENT_MAX);
  }
  cache.lastSeen = Date.now();
  persist();
}

function fmt(ts: number): string {
  if (!ts) return 'never';
  try {
    return new Date(ts).toLocaleString();
  } catch {
    return new Date(ts).toISOString();
  }
}

/** Render the `memory` command output. */
export function renderMemory(): string {
  const m = cache;
  if (m.visits === 0 && m.commandCount === 0) {
    return (
      `${D}This is your first session. MONAD has nothing remembered about you yet.${R}\r\n` +
      `${D}Anything it remembers will live only in this browser — never on a server.${R}\r\n`
    );
  }
  const recent = m.recentCommands.length ? m.recentCommands.join(', ') : '(none)';
  return (
    `${B}${A}MONAD MEMORY${R}  ${D}(stored only in this browser — nothing leaves your machine)${R}\r\n` +
    `  first seen   : ${fmt(m.firstSeen)}\r\n` +
    `  last seen    : ${fmt(m.lastSeen)}\r\n` +
    `  visits       : ${m.visits}\r\n` +
    `  commands run : ${m.commandCount}\r\n` +
    `  recent       : ${recent}\r\n` +
    `  backend      : ${backend}\r\n` +
    `${D}The kernel forgets every session. This memory is yours, on your hardware.${R}\r\n` +
    `${D}Run ${R}${A}forget${R}${D} to erase it.${R}\r\n`
  );
}

/** Wipe all client-side memory. */
export function forgetMemory(): string {
  cache = defaults();
  if (hasIDB()) {
    void (async () => {
      try {
        const db = await openDB();
        await new Promise<void>((resolve, reject) => {
          const tx = db.transaction(STORE, 'readwrite');
          tx.objectStore(STORE).delete(KEY);
          tx.oncomplete = () => resolve();
          tx.onerror = () => reject(tx.error);
        });
      } catch {
        /* ignore */
      }
    })();
  }
  try {
    localStorage.removeItem(LS_KEY);
  } catch {
    /* ignore */
  }
  return `${G}Memory erased. MONAD has forgotten you. Nothing left this machine.${R}\r\n`;
}

/** A short "welcome back" line for returning visitors, or null on first visit. */
export function welcomeBack(): string | null {
  if (cache.visits === 0) return null;
  const visitWord = cache.visits === 1 ? 'visit' : 'visits';
  return (
    `${D}Welcome back. Last seen ${fmt(cache.lastSeen)} · ${cache.visits} prior ${visitWord}.${R}\r\n` +
    `${D}Your machine remembered this, not a server. Type ${R}${A}memory${R}${D} to see what it knows, ${R}${A}forget${R}${D} to wipe it.${R}`
  );
}
