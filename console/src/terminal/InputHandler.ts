/**
 * InputHandler — captures keyboard input and maintains a line buffer + caret.
 *
 * It owns no DOM. On every change it reports the display string and caret
 * position via `render`; on submit/interrupt it asks the host to `commit` the
 * finished line into scrollback. This keeps editing logic separate from
 * rendering (the Screen draws the cursor).
 *
 * Line discipline: char echo, caret movement, history, Tab completion,
 * Ctrl+C / Ctrl+D / Ctrl+L, Enter to submit.
 */

export interface InputCallbacks {
  onSubmit: (line: string) => void;
  onTab: (prefix: string) => string[];
  onInterrupt: () => void;
  onClear: () => void;
  /** Live update of the editable line: display text + caret index. */
  render: (display: string, cursor: number) => void;
  /** Freeze the current input line (prefix + display) into scrollback. */
  commit: (display: string) => void;
  /** Write text into scrollback above the input line (e.g. Tab candidates). */
  print: (text: string) => void;
}

export class InputHandler {
  private buffer = '';
  private cursorPos = 0;
  private history: string[] = [];
  private historyIdx = -1;
  private echoEnabled = true;
  private enabled = false;
  private cb: InputCallbacks;

  constructor(cb: InputCallbacks) {
    this.cb = cb;
  }

  /** Handle a keydown event. Returns true if the key was handled. */
  handleKey(e: KeyboardEvent): boolean {
    if (!this.enabled) return false;

    // Ctrl+C — interrupt
    if (e.ctrlKey && e.key === 'c') {
      this.cb.commit(this.display() + '^C');
      this.reset();
      this.cb.onInterrupt();
      e.preventDefault();
      return true;
    }

    // Ctrl+D on empty line — exit
    if (e.ctrlKey && e.key === 'd') {
      if (this.buffer.length === 0) {
        this.cb.onSubmit('exit');
        e.preventDefault();
        return true;
      }
      return false;
    }

    // Ctrl+L — clear
    if (e.ctrlKey && e.key === 'l') {
      this.cb.onClear();
      e.preventDefault();
      return true;
    }

    // Tab — completion
    if (e.key === 'Tab') {
      const prefix = this.buffer.split(' ').pop() || '';
      const candidates = this.cb.onTab(prefix);
      if (candidates.length === 1) {
        this.insertAtCursor(candidates[0].slice(prefix.length));
      } else if (candidates.length > 1) {
        // Freeze the line, list candidates, then redraw the input line.
        this.cb.commit(this.display());
        this.cb.print(candidates.join('  '));
        this.update();
      }
      e.preventDefault();
      return true;
    }

    // Enter — submit
    if (e.key === 'Enter') {
      const line = this.buffer;
      this.cb.commit(this.display());
      this.reset();
      if (line.trim()) {
        this.history.push(line);
        if (this.history.length > 100) this.history.shift();
      }
      this.cb.onSubmit(line);
      e.preventDefault();
      return true;
    }

    // Backspace
    if (e.key === 'Backspace') {
      if (this.cursorPos > 0) {
        this.buffer = this.buffer.slice(0, this.cursorPos - 1) + this.buffer.slice(this.cursorPos);
        this.cursorPos--;
        this.update();
      }
      e.preventDefault();
      return true;
    }

    // Arrow Up — history back
    if (e.key === 'ArrowUp') {
      if (this.history.length === 0) return true;
      if (this.historyIdx === -1) this.historyIdx = this.history.length;
      this.historyIdx = Math.max(0, this.historyIdx - 1);
      this.buffer = this.history[this.historyIdx];
      this.cursorPos = this.buffer.length;
      this.update();
      e.preventDefault();
      return true;
    }

    // Arrow Down — history forward
    if (e.key === 'ArrowDown') {
      if (this.historyIdx === -1) return true;
      this.historyIdx = Math.min(this.history.length, this.historyIdx + 1);
      if (this.historyIdx >= this.history.length) {
        this.buffer = '';
        this.historyIdx = -1;
      } else {
        this.buffer = this.history[this.historyIdx];
      }
      this.cursorPos = this.buffer.length;
      this.update();
      e.preventDefault();
      return true;
    }

    // Caret movement
    if (e.key === 'ArrowLeft') {
      this.cursorPos = Math.max(0, this.cursorPos - 1);
      this.update();
      e.preventDefault();
      return true;
    }
    if (e.key === 'ArrowRight') {
      this.cursorPos = Math.min(this.buffer.length, this.cursorPos + 1);
      this.update();
      e.preventDefault();
      return true;
    }
    if (e.key === 'Home') {
      this.cursorPos = 0;
      this.update();
      e.preventDefault();
      return true;
    }
    if (e.key === 'End') {
      this.cursorPos = this.buffer.length;
      this.update();
      e.preventDefault();
      return true;
    }

    // Printable characters
    if (e.key.length === 1 && !e.ctrlKey && !e.metaKey) {
      this.insertAtCursor(e.key);
      e.preventDefault();
      return true;
    }

    return false;
  }

  /** Enable/disable input handling. Enabling redraws an empty line. */
  setEnabled(enabled: boolean): void {
    this.enabled = enabled;
  }

  /** Enable/disable echo (disable for password input — shows mask instead). */
  setEchoEnabled(enabled: boolean): void {
    this.echoEnabled = enabled;
  }

  /** Replace buffer + caret from an external source (mobile soft keyboard,
   *  whose hidden <input> is the source of truth while a touch device types). */
  setLine(text: string, cursor: number): void {
    if (!this.enabled) return;
    this.buffer = text;
    this.cursorPos = Math.max(0, Math.min(text.length, cursor));
    this.historyIdx = -1;
    this.update();
  }

  /** Reset the buffer and caret (keeps history). */
  reset(): void {
    this.buffer = '';
    this.cursorPos = 0;
    this.historyIdx = -1;
  }

  /** Force a redraw of the current line (used when (re)showing a prompt). */
  refresh(): void {
    this.update();
  }

  getBuffer(): string { return this.buffer; }

  // ─── private ───

  /** The visible representation of the buffer (masked when echo is off). */
  private display(): string {
    return this.echoEnabled ? this.buffer : '*'.repeat(this.buffer.length);
  }

  private insertAtCursor(text: string): void {
    if (!text) return;
    this.buffer = this.buffer.slice(0, this.cursorPos) + text + this.buffer.slice(this.cursorPos);
    this.cursorPos += text.length;
    this.update();
  }

  private update(): void {
    this.cb.render(this.display(), this.cursorPos);
  }
}
