/**
 * TTY — terminal emulator that ties the Screen, InputHandler, and kernel together.
 *
 * Responsibilities:
 *   - Own the active prompt prefix and render the editable line via the Screen.
 *   - Submit commands to the kernel and print their output.
 *   - Provide a clean `readLine()` API for the boot sequence (host-key, password)
 *     so it no longer reaches into InputHandler internals.
 */

import { Screen } from './Screen.ts';
import { InputHandler } from './InputHandler.ts';

export type PromptFn = () => string;

export class TTY {
  readonly screen: Screen;
  readonly input: InputHandler;
  private getPrompt: PromptFn;
  private executeCmd: (input: string) => string;
  private onExit: () => void;
  /** The prefix in front of the editable line (shell prompt or readLine prompt). */
  private currentPrefix = '';
  /** When set, the next submitted line resolves a pending readLine() instead of
   *  being executed as a command. */
  private pendingResolve: ((line: string) => void) | null = null;
  private running = true;

  constructor(opts: {
    element: HTMLElement;
    getPrompt: PromptFn;
    executeCmd: (input: string) => string;
    completions: (prefix: string) => string[];
    onExit: () => void;
  }) {
    this.screen = new Screen(opts.element);
    this.getPrompt = opts.getPrompt;
    this.executeCmd = opts.executeCmd;
    this.onExit = opts.onExit;

    this.input = new InputHandler({
      onSubmit: (line) => this.handleSubmit(line),
      onTab: (prefix) => opts.completions(prefix),
      onInterrupt: () => this.showPrompt(),
      onClear: () => { this.screen.clear(); this.showPrompt(); },
      render: (display, cursor) => this.screen.setInput(this.currentPrefix, display, cursor),
      commit: (display) => this.screen.commitInput(this.currentPrefix, display),
      print: (text) => this.screen.write(text + '\n'),
    });

    document.addEventListener('keydown', (e) => {
      if (!this.running) return;
      this.input.handleKey(e);
    });
  }

  /** Print committed output. */
  write(text: string): void {
    this.screen.write(text);
  }

  /** Show the interactive shell prompt and begin accepting input. */
  showPrompt(): void {
    this.screen.freshLine();
    this.currentPrefix = this.getPrompt();
    this.input.setEchoEnabled(true);
    this.input.reset();
    this.input.setEnabled(true);
    this.running = true;
    this.input.refresh();
  }

  /**
   * Read a single line under a custom prompt prefix (for the boot handshake).
   * Resolves with the entered text once the user presses Enter. With
   * `mask: true`, keystrokes display as `*`.
   */
  readLine(prefix: string, opts: { mask?: boolean } = {}): Promise<string> {
    return new Promise((resolve) => {
      this.currentPrefix = prefix;
      this.input.setEchoEnabled(!opts.mask);
      this.input.reset();
      this.input.setEnabled(true);
      this.running = true;
      this.pendingResolve = resolve;
      this.input.refresh();
    });
  }

  getBuffer(): string {
    return this.input.getBuffer();
  }

  /** Handle a submitted line — either resolve a readLine() or run a command. */
  private handleSubmit(line: string): void {
    // Boot handshake is waiting on this line.
    if (this.pendingResolve) {
      const resolve = this.pendingResolve;
      this.pendingResolve = null;
      this.input.setEchoEnabled(true);
      this.input.setEnabled(false);
      resolve(line);
      return;
    }

    if (line === 'exit' || line === 'logout') {
      this.running = false;
      this.input.setEnabled(false);
      this.screen.write('logout\n\n[connection closed.]\n');
      this.screen.clearInput();
      this.onExit();
      return;
    }

    if (line.trim()) {
      const output = this.executeCmd(line);
      if (output) this.screen.write(output);
    }

    this.showPrompt();
  }
}
