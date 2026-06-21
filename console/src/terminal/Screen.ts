/**
 * Screen — DOM-based terminal renderer.
 *
 * Model:
 *   - `lines`   : committed scrollback (complete lines, ANSI).
 *   - `current` : the open line being written to (no trailing newline yet).
 *   - `input`   : the live, editable region appended after `current` — a prompt
 *                 prefix plus the user's typed text, with a blinking block cursor
 *                 rendered at the caret. This is what makes the typing indicator
 *                 visible; output and input are never conflated.
 *
 * The kernel produces all colour/formatting; the Screen only converts ANSI to DOM.
 */

interface InputRegion {
  prefix: string;   // ANSI prompt text (e.g. the shell prompt or "Password:")
  text: string;     // already-masked display text (caller handles echo)
  cursor: number;   // caret index within `text`
  active: boolean;  // whether to draw the blinking cursor
}

export class Screen {
  private el: HTMLElement;
  private lines: string[] = [];
  private current = '';
  private input: InputRegion = { prefix: '', text: '', cursor: 0, active: false };
  private readonly maxLines = 1000;

  constructor(element: HTMLElement) {
    this.el = element;
  }

  /** Write committed output. Splits on newlines; text without a trailing
   *  newline stays on the open `current` line. Honours screen-clear (ESC[2J). */
  write(text: string): void {
    if (!text) return;

    // Full-screen clear (e.g. the `clear` command) — wipe and keep any tail.
    const clearIdx = text.lastIndexOf('\x1b[2J');
    if (clearIdx !== -1) {
      this.lines = [];
      this.current = '';
      text = text.slice(clearIdx + 4).replace('\x1b[H', '');
      if (!text) { this.render(); return; }
    }

    // Normalise line endings; a lone '\r' returns to start of the current line.
    text = text.replace(/\r\n/g, '\n').replace(/\r/g, '');

    for (const ch of text) {
      if (ch === '\n') {
        this.lines.push(this.current);
        this.current = '';
      } else if (ch === '') {
        this.current = '';
      } else {
        this.current += ch;
      }
    }

    this.trim();
    this.render();
  }

  /** Set / update the live editable input region (prefix + text + caret). */
  setInput(prefix: string, text: string, cursor: number): void {
    this.input = { prefix, text, cursor, active: true };
    this.render();
  }

  /** Commit the current input line (prefix + final text) into scrollback. */
  commitInput(prefix: string, text: string): void {
    this.write(prefix + text + '\n');
    this.input.active = false;
    this.render();
  }

  /** Hide the editable region without committing (e.g. after exit). */
  clearInput(): void {
    this.input.active = false;
    this.render();
  }

  /** If the open line has content, push it to scrollback so the next thing
   *  written starts fresh (used before showing a prompt). */
  freshLine(): void {
    if (this.current.length > 0) {
      this.lines.push(this.current);
      this.current = '';
    }
  }

  /** Clear the entire terminal. */
  clear(): void {
    this.lines = [];
    this.current = '';
    this.input.active = false;
    this.el.innerHTML = '';
    this.el.scrollTop = 0;
  }

  // ─── internal ───

  private trim(): void {
    if (this.lines.length > this.maxLines) {
      this.lines = this.lines.slice(-this.maxLines);
    }
  }

  private render(): void {
    const rendered = this.lines.map(line => ansiToInlineHtml(line));

    // The bottom (open) line: current output tail + the live input region.
    let bottom = ansiToInlineHtml(this.current);
    if (this.input.active) {
      bottom += ansiToInlineHtml(this.input.prefix) + renderInput(this.input);
    }
    rendered.push(bottom);

    this.el.innerHTML = rendered.map(h => h || '').join('<br>');
    this.el.scrollTop = this.el.scrollHeight;
  }
}

/** Render the editable text with a blinking block cursor at the caret. */
function renderInput(input: InputRegion): string {
  const { text, cursor } = input;
  const before = escapeHtml(text.slice(0, cursor));
  const atChar = text[cursor];
  const at = atChar ? escapeHtml(atChar) : ' ';
  const after = escapeHtml(text.slice(cursor + 1));
  return `${before}<span class="cursor">${at}</span>${after}`;
}

/** Strip ANSI CSI sequences that aren't SGR (cursor moves, erase) and OSC-8
 *  terminators we handle separately. */
function stripAnsiCsi(text: string): string {
  return text.replace(/\x1b\[[\d;]*[A-HJ-UX-Za-hj-ux-z]/g, '');
}

// ─── ANSI to HTML converter ───

interface StyleState {
  bold: boolean;
  dim: boolean;
  italic: boolean;
  underline: boolean;
  fg: string | null;
  bg: string | null;
}

const ANSI_COLORS: Record<number, string> = {
  0: '#c6c6c6', 1: '#d47766', 2: '#87d96c', 3: '#e6b91e',
  4: '#78dce8', 5: '#ab9df2', 6: '#56b6c2', 7: '#c6c6c6',
};

// 8-bit (256-colour) palette entries the kernel actually uses.
const ANSI_256: Record<number, string> = {
  81: '#78dce8', 113: '#87d96c', 131: '#d47766',
  214: '#e6b91e', 220: '#ffd866', 240: '#585858', 251: '#c6c6c6',
};

function escapeHtml(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

/** Convert one ANSI line to inline HTML (no trailing <br>). */
function ansiToInlineHtml(text: string): string {
  text = stripAnsiCsi(text);
  if (!text) return '';

  const parts = text.split(/(\x1b\[[\d;]*m|\x1b\]8;;[^\x1b]*\x1b\\)/g);
  const out: string[] = [];
  let state: StyleState = resetState();
  let linkOpen = false;

  for (const part of parts) {
    if (!part) continue;

    // OSC-8 hyperlink open/close
    if (part.startsWith('\x1b]8;;') && part.endsWith('\x1b\\')) {
      const url = part.slice(5, -2);
      if (url) {
        out.push(`<a href="${escapeHtml(url)}" target="_blank" rel="noopener" style="color:#78dce8;text-decoration:underline;">`);
        linkOpen = true;
      } else if (linkOpen) {
        out.push('</a>');
        linkOpen = false;
      }
      continue;
    }

    // SGR code
    if (part.startsWith('\x1b[') && part.endsWith('m')) {
      const params = part.slice(2, -1);
      if (!params) { state = resetState(); continue; }
      const codes = params.split(';').map(c => parseInt(c, 10));
      for (let i = 0; i < codes.length; i++) {
        const code = codes[i];
        if (isNaN(code)) continue;
        // 8-bit colour: 38;5;N (fg) or 48;5;N (bg)
        if ((code === 38 || code === 48) && codes[i + 1] === 5) {
          const n = codes[i + 2];
          const colour = ANSI_256[n] ?? null;
          if (code === 38) state.fg = colour; else state.bg = colour;
          i += 2;
          continue;
        }
        switch (code) {
          case 0:  state = resetState(); break;
          case 1:  state.bold = true; break;
          case 2:  state.dim = true; break;
          case 3:  state.italic = true; break;
          case 4:  state.underline = true; break;
          case 22: state.bold = false; state.dim = false; break;
          case 23: state.italic = false; break;
          case 24: state.underline = false; break;
          default:
            if (code >= 30 && code <= 37) state.fg = ANSI_COLORS[code];
            else if (code >= 40 && code <= 47) state.bg = ANSI_COLORS[code - 10];
            else if (code >= 90 && code <= 97) state.fg = ANSI_COLORS[code - 90];
            else if (code >= 100 && code <= 107) state.bg = ANSI_COLORS[code - 100];
            break;
        }
      }
      continue;
    }

    // Plain text — wrap in a styled span when needed.
    const escaped = escapeHtml(part);
    const styles = buildStyle(state);
    out.push(styles ? `<span style="${styles}">${escaped}</span>` : escaped);
  }

  if (linkOpen) out.push('</a>');
  return out.join('');
}

function resetState(): StyleState {
  return { bold: false, dim: false, italic: false, underline: false, fg: null, bg: null };
}

function buildStyle(s: StyleState): string {
  const parts: string[] = [];
  if (s.bold) parts.push('font-weight:700');
  if (s.dim) parts.push('opacity:0.7');
  if (s.italic) parts.push('font-style:italic');
  if (s.underline) parts.push('text-decoration:underline');
  if (s.fg) parts.push(`color:${s.fg}`);
  if (s.bg) parts.push(`background:${s.bg}`);
  return parts.join(';');
}
