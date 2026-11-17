use std::fs;
use std::time::Duration;

use clap::Parser;
use zellij_utils::consts::{ipc_connect, session_info_folder_for_session, ZELLIJ_SOCK_DIR};
use zellij_utils::envs;
use zellij_utils::sessions::{get_resurrectable_sessions, read_live_session_states_default_dirs};

const DEFAULT_MAX_AGE_SECS: u64 = 7 * 24 * 60 * 60;

#[derive(Parser, Debug)]
#[command(name = "zellij-prune", version, about = "Prune old Zellij sessions")]
struct Cli {
    /// Maximum session age before the session is pruned (e.g. "2w", "30m", "1d", "3600")
    #[arg(long, default_value = "604800")]
    max_age: String,

    /// Session-name prefix that marks a session as protected (never pruned)
    #[arg(long, default_value = "@")]
    prefix: String,

    /// Only print the sessions that would be pruned, without killing them
    #[arg(long)]
    dry_run: bool,
}

fn parse_duration_secs(input: &str) -> Option<u64> {
    let input = input.trim().to_lowercase();
    if let Ok(secs) = input.parse::<u64>() {
        return Some(secs);
    }

    let mut remaining: &str = &input;
    let mut total_secs: u64 = 0;

    while !remaining.is_empty() {
        let num_end = remaining
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(remaining.len());

        let num: f64 = remaining[..num_end].parse().ok()?;
        let unit = remaining[num_end..].chars().next()?;

        let mult: f64 = match unit {
            's' => 1.0,
            'm' => 60.0,
            'h' => 3600.0,
            'd' => 86400.0,
            'w' => 604800.0,
            _ => return None,
        };

        total_secs += (num * mult) as u64;
        remaining = &remaining[num_end + 1..];
    }

    Some(total_secs)
}

fn prune_session(name: &str) {
    let path = ZELLIJ_SOCK_DIR.join(name);
    if let Ok(stream) = ipc_connect(&path) {
        #[cfg(not(windows))]
        {
            use zellij_utils::ipc::{ClientToServerMsg, IpcSenderWithContext};
            let _ = IpcSenderWithContext::<ClientToServerMsg>::new(stream)
                .send_client_msg(ClientToServerMsg::KillSession);
        }

        #[cfg(windows)]
        {
            use zellij_utils::consts::ipc_connect_reply;
            use zellij_utils::ipc::{
                ClientToServerMsg, IpcReceiverWithContext, IpcSenderWithContext, ServerToClientMsg,
            };
            let reply = ipc_connect_reply(&path);
            let _ = IpcSenderWithContext::<ClientToServerMsg>::new(stream)
                .send_client_msg(ClientToServerMsg::KillSession);
            if let Ok(reply_stream) = reply {
                let mut receiver = IpcReceiverWithContext::new(reply_stream);
                let _ = receiver.recv_server_msg();
            }
        }
    }
    let _ = fs::remove_dir_all(session_info_folder_for_session(name));
}

fn main() {
    let cli = Cli::parse();
    let max_age_secs = parse_duration_secs(&cli.max_age).unwrap_or(DEFAULT_MAX_AGE_SECS);
    let max_age = Duration::from_secs(max_age_secs);
    let prefix = cli.prefix.trim().to_string();
    let dry_run = cli.dry_run;

    let current_session = envs::get_session_name().ok();
    let current_name = current_session.as_deref().unwrap_or("");

    let live_sessions = read_live_session_states_default_dirs(current_name);

    let mut targets: Vec<String> = Vec::new();
    for (name, info) in live_sessions {
        if !info.is_current_session
            && info.connected_clients == 0
            && !name.starts_with(prefix.as_str())
            && info.creation_time > max_age
        {
            targets.push(name);
        }
    }

    for (name, age) in get_resurrectable_sessions() {
        if !name.starts_with(prefix.as_str()) && age > max_age {
            targets.push(name);
        }
    }

    if targets.is_empty() {
        eprintln!("No sessions to prune.");
        return;
    }

    if dry_run {
        eprintln!("Would prune {} sessions.", targets.len());
        return;
    }

    for name in &targets {
        prune_session(name);
    }
    eprintln!("Pruned {} sessions.", targets.len());
}
