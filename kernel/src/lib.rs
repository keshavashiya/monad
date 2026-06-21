//! MONAD kernel — the pure, host-agnostic core.
//!
//! All identity logic lives here: command dispatch, the virtual OS, the vault,
//! and terminal rendering. The kernel has no knowledge of its runtime. Each
//! adapter (browser, CLI, WASI, MCP, daemon) constructs a [`Kernel`] with a
//! [`Host`] and renders whatever [`Kernel::execute`] returns.

mod commands;
mod os;
mod query;
mod terminal;
mod vault;
pub mod host;

pub use host::Host;
pub use query::TOPICS;
use os::session::Session;

/// Reproducible SHA-256 of the embedded vault (`vault/profile.json`), computed
/// at build time. Reproduce with `shasum -a 256 vault/profile.json`. Surfaced
/// to humans via the `verify` command and to adapters that want to display it.
pub const BUILD_HASH: &str = env!("MONAD_VAULT_SHA256");

/// The MONAD version — the kernel crate's own version, the single source of
/// truth across every adapter. Lives in the binary (not the vault) so it can
/// never drift from the code that's actually running.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// A booted MONAD instance. Holds the embedded vault and the (only) mutable
/// session state. Construct one per connection; drop it to forget everything.
pub struct Kernel {
    session: Session,
}

impl Kernel {
    /// Boot the kernel with a host adapter that supplies the wall clock.
    pub fn new(host: Box<dyn Host>) -> Self {
        let vault = vault::Vault::load();
        Self {
            session: Session::new(vault, host),
        }
    }

    /// Execute a single command line and return ANSI-formatted output.
    /// Errors are rendered inline (red) so adapters can print the string verbatim.
    pub fn execute(&mut self, input: &str) -> String {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        match commands::dispatch(trimmed, &mut self.session) {
            Ok(output) => {
                self.session.add_history(trimmed.to_string());
                output
            }
            Err(msg) => format!("\x1b[38;5;131m{}\x1b[0m\r\n", msg),
        }
    }

    /// Tab-completion candidates for a command prefix. Stateless.
    pub fn completions(prefix: &str) -> Vec<String> {
        commands::completions(prefix)
    }

    /// The current working directory of this session.
    pub fn cwd(&self) -> &str {
        &self.session.cwd
    }

    /// Tell the kernel how wide the adapter's terminal is (character columns),
    /// so tables can fill the screen. Call on connect and on resize.
    pub fn set_cols(&mut self, cols: usize) {
        self.session.set_cols(cols);
    }

    /// The MONAD version (from the kernel crate, via [`VERSION`]). Shown across
    /// every adapter — help, neofetch, verify, CLI.
    pub fn version(&self) -> &str {
        VERSION
    }

    /// The canonical home directory (the `~` target) for this session.
    pub fn home(&self) -> &str {
        &self.session.home
    }

    /// The login short-name (`<user>@monad`) for shell prompts, derived from
    /// the vault rather than hardcoded in each adapter (CLI, console).
    pub fn user(&self) -> &str {
        self.session.user()
    }

    /// The identity's display name (from the vault), for adapters that
    /// introduce the author — e.g. the MCP server's tool descriptions.
    pub fn name(&self) -> &str {
        &self.session.vault.identity.name
    }

    /// A named link from the vault (`resume`, `meeting`, `github`, …), or an
    /// empty string if absent. Lets adapters act on a URL (open it, download
    /// it) without parsing rendered output. The kernel stays pure: it only
    /// reports the URL; the I/O lives in the adapter.
    pub fn link(&self, name: &str) -> &str {
        let l = &self.session.vault.links;
        match name {
            "email" => &l.email,
            "github" => l.github.as_deref().unwrap_or(""),
            "linkedin" => l.linkedin.as_deref().unwrap_or(""),
            "twitter" => l.twitter.as_deref().unwrap_or(""),
            "website" => l.website.as_deref().unwrap_or(""),
            "resume" => l.resume.as_deref().unwrap_or(""),
            "meeting" => l.meeting.as_deref().unwrap_or(""),
            _ => "",
        }
    }

    /// Structured data plane: return JSON for a topic (for programs/agents).
    /// Valid topics are listed in [`TOPICS`]; unknown topics return an error
    /// object rather than failing.
    pub fn query(&self, topic: &str) -> serde_json::Value {
        query::query(&self.session, topic)
    }

    /// Answer a natural-language-ish question with relevant facts as plain text.
    /// Deterministic keyword routing — no LLM. The calling agent supplies the
    /// language model; the kernel supplies ground truth.
    pub fn ask(&self, question: &str) -> String {
        query::ask(&self.session, question)
    }
}
