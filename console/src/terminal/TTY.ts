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
  /** Off-screen, focusable input that exists solely to open the mobile soft
   *  keyboard (nothing else on the page is focusable) and to receive its input
   *  events, which fire reliably where keydown does not. */
  private keyInput!: HTMLInputElement;

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

    this.setupKeyboard(opts.element);
  }

  /** Build the hidden input and route both hardware (keydown) and soft-keyboard
   *  (input event) typing into the line buffer. */
  private setupKeyboard(element: HTMLElement): void {
    const kbd = document.createElement('input');
    kbd.setAttribute('autocapitalize', 'none');
    kbd.setAttribute('autocomplete', 'off');
    kbd.setAttribute('autocorrect', 'off');
    kbd.setAttribute('spellcheck', 'false');
    kbd.setAttribute('aria-hidden', 'true');
    kbd.tabIndex = -1;
    // Off-screen but focusable. font-size:16px stops iOS zoom-on-focus;
    // caret-color:transparent hides its native caret (the Screen draws ours).
    kbd.style.cssText =
      'position:fixed;bottom:0;left:0;width:1px;height:1px;opacity:0;' +
      'border:0;padding:0;margin:0;font-size:16px;z-index:-1;caret-color:transparent;';
    document.body.appendChild(kbd);
    this.keyInput = kbd;

    // Tap anywhere on the terminal → focus the hidden input so the soft
    // keyboard opens (iOS only opens it from inside a user gesture).
    element.addEventListener('pointerdown', () => { if (this.running) kbd.focus(); });

    // Hardware keyboards: keydown carries the key; handleKey preventDefaults
    // everything it handles, so the hidden input's value never changes and no
    // input event fires (no double-typing on desktop).
    document.addEventListener('keydown', (e) => {
      if (!this.running) return;
      this.input.handleKey(e);
    });

    // Soft keyboards: printable keys arrive only as input events with the
    // hidden input as source of truth. Mirror its full value + caret into the
    // line buffer; backspace/selection/paste all reflect here for free.
    kbd.addEventListener('input', () => {
      if (!this.running) return;
      // Some soft keyboards deliver Enter as a newline in the value rather than
      // a keydown — treat it as submit.
      if (kbd.value.includes('\n')) {
        this.input.setLine(kbd.value.replace(/\n.*/s, ''), kbd.value.indexOf('\n'));
        this.input.handleKey(new KeyboardEvent('keydown', { key: 'Enter' }));
        return;
      }
      this.input.setLine(kbd.value, kbd.selectionStart ?? kbd.value.length);
    });
  }

  /** Clear the hidden input so the next line starts empty (its value persists
   *  across submits otherwise — Enter doesn't clear it). */
  private clearKeyInput(): void {
    this.keyInput.value = '';
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
    this.clearKeyInput();
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
      this.clearKeyInput();
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
    // Enter cleared the line buffer but not the hidden input — clear it so the
    // next line doesn't inherit the submitted text.
    this.clearKeyInput();

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
