/**
 * Boot sequence — simulates connecting to a remote MONAD host.
 *
 * Stages:
 *   1. Black screen (momentary)
 *   2. Bootloader banner
 *   3. Kernel boot messages (dmesg)
 *   4. SSH handshake: host key check, password
 *   5. MOTD + shell prompt
 */

import type { TTY } from './terminal/TTY.ts';
import { acceptHostKey, isHostKeyAccepted } from './state.ts';
import { loadKernel, getBuildHash, getVersion, getUser } from './kernel.ts';
import { getMemory, rememberHostKey, welcomeBack } from './memory.ts';

function motd(buildHash: string, version: string): string {
  const short = buildHash ? buildHash.slice(0, 16) : 'unknown';
  const v = version || '—';
  return (
`\x1b[1m\x1b[38;5;214mMONAD v${v}\x1b[0m \x1b[2m— compiled identity kernel  vault:${short}…\x1b[0m
\x1b[2m────────────────────────────────────────────────────────────────────────────────\x1b[0m
\x1b[2mOne core, many adapters. Ships complete. Every boot deterministic.\x1b[0m
\x1b[2mType \x1b[0m\x1b[38;5;214mhelp\x1b[0m\x1b[2m for commands, or \x1b[0m\x1b[38;5;214mverify\x1b[0m\x1b[2m to check this build.\x1b[0m`
  );
}

export class BootSequence {
  private tty: TTY;
  private aborted = false;

  constructor(tty: TTY) {
    this.tty = tty;
  }

  async run(): Promise<void> {
    await this.stageBlackScreen();
    if (this.aborted) return;

    await this.stageBootloader();
    if (this.aborted) return;

    await this.stageKernelBoot();

    // Load the kernel before the handshake so the login name shown at the
    // password prompt comes from the vault, not a placeholder.
    await loadKernel();

    await this.stageSshHandshake();
    if (this.aborted) return;

    await this.stageMotd();
    this.tty.showPrompt();
  }

  /** Stage 1: Brief black screen */
  private async stageBlackScreen(): Promise<void> {
    this.tty.screen.write('\x1b[2J\x1b[H');
    await this.delay(100);
  }

  /** Stage 2: Bootloader banner with auto-boot timer */
  private async stageBootloader(): Promise<void> {
    const buildTime = '2026-06-13T00:00:00Z';

    this.tty.screen.write(
      `  \x1b[1m\x1b[38;5;214mMONAD BOOTLOADER\x1b[0m\n` +
      `  \x1b[2mBuild: ${buildTime}\x1b[0m\n` +
      `  \x1b[2mFirmware: reproducible (run \x1b[0m\x1b[38;5;214mverify\x1b[0m\x1b[2m after boot)\x1b[0m\n` +
      `  \x1b[2m──────────────────────────────────────────\x1b[0m\n` +
      `  \x1b[2mPress any key to boot, or waiting...\x1b[0m`
    );

    await this.waitForKeyOrTimeout(800);
    if (this.aborted) return;
  }

  /** Stage 3: Kernel boot messages */
  private async stageKernelBoot(): Promise<void> {
    const messages = [
      '[    0.000000] MONAD (one core, many adapters)',
      '[    0.000412] Kernel loaded at 0x0000, host interface attached',
      '[    0.000889] Vault integrity: reproducible sha256 verified',
      '[    0.001201] Session initialized. Console attached.',
      '[    0.001543] MONAD ready.',
    ];

    this.tty.screen.write('\n');
    for (const msg of messages) {
      if (this.aborted) return;
      this.tty.screen.write(`  \x1b[2m${msg}\x1b[0m\n`);
      await this.delay(40);
    }
    await this.delay(120);
  }

  /** Stage 4: SSH handshake simulation */
  private async stageSshHandshake(): Promise<void> {
    this.tty.screen.write('\n');
    // A returning visitor (remembered by this browser) skips the host-key prompt.
    if (isHostKeyAccepted() || getMemory().hostKeyAccepted) {
      this.tty.screen.write(`  \x1b[2mHost key accepted. Authenticating...\x1b[0m\n`);
      await this.delay(200);
      this.tty.screen.write(`  \x1b[38;5;113mAuthentication successful.\x1b[0m\n`);
      return;
    }

    await this.stageHostKey();
    if (this.aborted) return;
    await this.stagePassword();
  }

  /** Host key verification */
  private async stageHostKey(): Promise<void> {
    this.tty.screen.write(
      `  \x1b[2mAuthenticity of host 'monad (127.0.0.1)' can't be established.\x1b[0m\n` +
      `  \x1b[2mED25519 key fingerprint \x1b[0m\x1b[38;5;214mSHA256:7KMjBqF...pCg\x1b[0m\n`
    );

    const response = await this.tty.readLine(
      `  \x1b[2mAre you sure you want to continue connecting? (yes/no)\x1b[0m `
    );

    if (response.toLowerCase() === 'yes' || response.toLowerCase() === 'y') {
      this.tty.screen.write(`  \x1b[2mHost key accepted.\x1b[0m\n`);
      acceptHostKey();
      rememberHostKey();
    } else {
      this.tty.screen.write(`\x1b[38;5;131mHost key verification failed. Connection closed.\x1b[0m\n`);
      this.aborted = true;
    }
  }

  /** Password prompt (any input accepted — this is a public demo host) */
  private async stagePassword(): Promise<void> {
    await this.tty.readLine(`  \x1b[2m(${getUser()}@monad) Password:\x1b[0m `, { mask: true });
    this.tty.screen.write(`  \x1b[2mAuthenticating...\x1b[0m\n`);
    await this.delay(200);
    this.tty.screen.write(`  \x1b[38;5;113mAuthentication successful.\x1b[0m\n`);
  }

  /** Stage 5: MOTD (plus a "welcome back" line for returning visitors) */
  private async stageMotd(): Promise<void> {
    this.tty.screen.write(`\n${motd(getBuildHash(), getVersion())}\n`);
    const back = welcomeBack();
    if (back) {
      this.tty.screen.write(`${back}\n`);
    }
  }

  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  private waitForKeyOrTimeout(timeoutMs: number): Promise<void> {
    return new Promise((resolve) => {
      let resolved = false;
      const handler = () => {
        if (!resolved) {
          resolved = true;
          document.removeEventListener('keydown', handler);
          resolve();
        }
      };
      document.addEventListener('keydown', handler);
      setTimeout(() => {
        if (!resolved) {
          resolved = true;
          document.removeEventListener('keydown', handler);
          resolve();
        }
      }, timeoutMs);
    });
  }
}
