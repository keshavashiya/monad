use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The compiled-in identity vault — embedded at build time via include_str!.
/// This is the "root filesystem" of MONAD. Everything the kernel knows
/// is defined here.

#[derive(Debug, Serialize, Deserialize)]
pub struct Vault {
    pub meta: Meta,
    pub identity: Identity,
    pub roles: Vec<Role>,
    pub stacks: Vec<Stack>,
    pub systems: Vec<System>,
    pub projects: Vec<Project>,
    pub links: Links,
    pub filesystem: HashMap<String, FsEntry>,
    pub processes: Vec<Process>,
    pub bootlog: Vec<LogEntry>,
    pub fortune: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Meta {
    pub kernel: String,
    // Version intentionally lives in the kernel crate (monad_kernel::VERSION),
    // not the vault, so it can never drift from the running code.
    pub build: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Identity {
    pub name: String,
    pub handle: String,
    pub title: String,
    pub location: Option<String>,
    pub timezone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Role {
    pub title: String,
    pub org: String,
    pub tenure: String,
    pub focus: String,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stack {
    pub name: String,
    pub proficiency: String,
    pub years: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct System {
    pub name: String,
    pub architecture: String,
    pub description: String,
    pub links: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub status: String,
    pub description: String,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Links {
    pub email: String,
    pub github: Option<String>,
    pub linkedin: Option<String>,
    pub twitter: Option<String>,
    pub website: Option<String>,
    pub resume: Option<String>,
    pub meeting: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FsEntry {
    #[serde(rename = "type")]
    pub entry_type: String,
    pub content: Option<String>,
    pub size: Option<u64>,
    pub permissions: Option<String>,
    pub owner: Option<String>,
    pub group: Option<String>,
    pub modified: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Process {
    pub pid: i32,
    pub ppid: i32,
    pub user: Option<String>,
    pub command: String,
    pub state: String,
    pub cpu: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub message: String,
}

impl Vault {
    pub fn load() -> Self {
        let data = include_str!("../../vault/profile.json");
        serde_json::from_str(data).expect("Failed to parse vault/profile.json")
    }
}
