// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025-2026 Sreehari Anil and project contributors

use crate::ipc::protocol::constants;

/// Returns the current user's UID on Unix-based systems.
#[cfg(unix)]
fn current_uid() -> u32 {
    unsafe { libc::getuid() }
}

/// Discovers potential base directories where Discord IPC sockets may exist on Unix systems.
#[cfg(unix)]
pub fn candidate_ipc_directories() -> Vec<String> {
    let env_keys = ["XDG_RUNTIME_DIR", "TMPDIR", "TMP", "TEMP", "tmp"];
    let mut directories = Vec::new();

    for key in &env_keys {
        if let Ok(dir) = std::env::var(key) {
            directories.push(dir.clone());

            // Check specialized paths if XDG_RUNTIME_DIR is set
            if key == &"XDG_RUNTIME_DIR" {
                // Flatpak
                directories.push(format!("{}/app/com.discordapp.Discord", dir));

                // Snap (can be snap.discord, snap.discord-canary, etc.)
                if let Ok(entries) = std::fs::read_dir(&dir) {
                    for entry in entries.flatten() {
                        if let Ok(name) = entry.file_name().into_string() {
                            if name.starts_with("snap.discord") {
                                if let Ok(metadata) = entry.metadata() {
                                    if metadata.is_dir() {
                                        directories.push(format!("{}/{}", dir, name));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback to /run/user/{uid} if directories list is still thin or to ensure coverage
    let uid = current_uid();
    let run_user_dir = format!("/run/user/{}", uid);
    if !directories.contains(&run_user_dir) {
        directories.push(run_user_dir.clone());
    }

    // Fallback specialized paths for /run/user/{uid}
    let flatpak_fallback = format!("/run/user/{}/app/com.discordapp.Discord", uid);
    if !directories.contains(&flatpak_fallback) {
        directories.push(flatpak_fallback);
    }

    // Snap fallback scan
    let base_run_user = format!("/run/user/{}", uid);
    if let Ok(entries) = std::fs::read_dir(&base_run_user) {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                if name.starts_with("snap.discord") {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_dir() {
                            let path = format!("{}/{}", base_run_user, name);
                            if !directories.contains(&path) {
                                directories.push(path);
                            }
                        }
                    }
                }
            }
        }
    }

    directories
}

/// Returns a list of all potential Discord IPC socket/pipe paths to check.
#[cfg(unix)]
pub fn get_socket_paths() -> Vec<String> {
    get_socket_paths_with_limit(constants::MAX_IPC_SOCKETS)
}

/// Returns a list of all potential Discord IPC socket paths to check with a custom scan limit.
#[cfg(unix)]
pub fn get_socket_paths_with_limit(max_sockets: u8) -> Vec<String> {
    let mut paths = Vec::new();
    for dir in candidate_ipc_directories() {
        for i in 0..max_sockets {
            paths.push(format!("{}/{}{}", dir, constants::IPC_SOCKET_PREFIX, i));
        }
    }
    paths
}

#[cfg(windows)]
pub fn get_pipe_paths() -> Vec<String> {
    get_pipe_paths_with_limit(constants::MAX_IPC_SOCKETS)
}

/// Returns a list of all potential Discord IPC pipe paths to check with a custom scan limit.
#[cfg(windows)]
pub fn get_pipe_paths_with_limit(max_sockets: u8) -> Vec<String> {
    let mut paths = Vec::new();
    for i in 0..max_sockets {
        paths.push(format!(r"\\.\pipe\discord-ipc-{}", i));
    }
    paths
}
