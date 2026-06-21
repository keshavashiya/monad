/**
 * MONAD Console — main entry point.
 *
 * Initialises the terminal, loads the kernel, and starts the boot sequence.
 */

import './style.css';
import { TTY } from './terminal/TTY.ts';
import { BootSequence } from './boot.ts';
import { execute, completions, getCwd, getHome, getUser, getLink, setCols } from './kernel.ts';
import { initMemory, beginVisit, recordCommand, renderMemory, forgetMemory } from './memory.ts';

// Commands handled by the console adapter, not the kernel: they concern
// client-side memory, which is the browser's state, not the kernel's.
const CLIENT_COMMANDS = ['memory', 'forget'];

async function main() {
  const container = document.getElementById('terminal');
  if (!container) {
    throw new Error('Terminal container not found');
  }

  // Clear any existing content
  container.innerHTML = '';

  // Measure how many monospace characters fit across the terminal, so the
  // kernel can size tables to the viewport. Recomputed on resize.
  const measureCols = (): number => {
    const probe = document.createElement('span');
    probe.style.cssText = 'position:absolute;visibility:hidden;white-space:pre';
    probe.textContent = '0'.repeat(100);
    container.appendChild(probe);
    const charWidth = probe.getBoundingClientRect().width / 100;
    container.removeChild(probe);
    const cs = getComputedStyle(container);
    const padX = parseFloat(cs.paddingLeft) + parseFloat(cs.paddingRight);
    const avail = container.clientWidth - padX;
    if (!charWidth || avail <= 0) return 80;
    return Math.max(24, Math.floor(avail / charWidth));
  };
  const reportCols = () => setCols(measureCols());
  window.addEventListener('resize', reportCols);

  // Load this browser's persisted memory before booting (for "welcome back").
  await initMemory();

  // Prompt function — called after every command
  const getPrompt = (): string => {
    const cwd = getCwd();
    const home = getHome();
    const dir = cwd === home
      ? '~'
      : cwd.startsWith(home + '/') ? '~' + cwd.slice(home.length) : cwd;
    return `\x1b[38;5;214m[${getUser()}@monad ${dir}]\x1b[0m$ `;
  };

  // Execute function — called on every command
  const executeCmd = (input: string): string => {
    const cmd = input.trim().split(/\s+/)[0];
    // Client-side memory commands are handled here, never by the kernel.
    if (cmd === 'memory') return renderMemory();
    if (cmd === 'forget') return forgetMemory();
    recordCommand(input.trim());
    const output = execute(input);
    // Browser-only side effect: `resume`/`meeting` also open the vault URL in a
    // new tab. The kernel only *reports* the link (so the CLI/MCP/npx adapters
    // still work); the actual I/O belongs to this adapter.
    if (cmd === 'resume' || cmd === 'meeting') {
      const url = getLink(cmd);
      if (url) window.open(url, '_blank', 'noopener');
    }
    return output;
  };

  // Completions function — called on Tab (kernel commands + client commands)
  const completionsFn = (prefix: string): string[] => {
    const extra = CLIENT_COMMANDS.filter(c => c.startsWith(prefix));
    return [...completions(prefix), ...extra];
  };

  // On exit — show closing message
  const onExit = () => {
    // Terminal shows "connection closed" then stops
  };

  // Create TTY
  const tty = new TTY({
    element: container,
    getPrompt,
    executeCmd,
    completions: completionsFn,
    onExit,
  });

  // Start boot sequence
  const boot = new BootSequence(tty);
  try {
    await boot.run();
    // Kernel is loaded now — report the terminal width so tables fill it.
    reportCols();
    // Record this visit only after a successful boot (so the count is real).
    beginVisit();
  } catch (err) {
    console.error('[monad] boot failed:', err);
    container.innerHTML = `
      <span style="color:#d47766">MONAD boot failed.</span>
      <br>
      <span style="opacity:0.7">${err instanceof Error ? err.message : String(err)}</span>
    `;
  }
}

// Wait for DOM
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', () => { void main(); });
} else {
  void main();
}
