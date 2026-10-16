//! Path traversal protection guard.

use std::path::{Component, Path, PathBuf};

use crate::errors::PolicyError;

pub struct PathGuard;

impl PathGuard {
    /// Resolve and sanitize a relative path strictly within workspace_root.
    /// Rejects parent traversal (`..`), absolute paths, null bytes, colons/ADS, and UNC paths.
    pub fn resolve_within_workspace(
        workspace_root: &Path,
        relative_path: &str,
    ) -> Result<PathBuf, PolicyError> {
        let trimmed = relative_path.trim();
        if trimmed.is_empty() {
            return Err(PolicyError::InvalidPath("Path cannot be empty".to_string()));
        }

        // Check for forbidden characters (null byte, Windows ADS)
        if trimmed.contains('\0') {
            return Err(PolicyError::InvalidPath(
                "Null bytes not allowed in path".to_string(),
            ));
        }
        if trimmed.contains(':') {
            return Err(PolicyError::InvalidPath(
                "Colons or alternate data streams not allowed in path".to_string(),
            ));
        }

        let lower = trimmed.to_lowercase();
        if lower.contains("%2e") || lower.contains("%2f") || lower.contains("%5c") {
            return Err(PolicyError::PathTraversal {
                path: trimmed.to_string(),
            });
        }

        let p = Path::new(trimmed);

        // Disallow absolute paths and root separators
        if p.is_absolute() || trimmed.starts_with('/') || trimmed.starts_with('\\') {
            return Err(PolicyError::AbsolutePathDisallowed(trimmed.to_string()));
        }

        let mut clean_components = Vec::new();
        for component in p.components() {
            match component {
                Component::Prefix(_) | Component::RootDir => {
                    return Err(PolicyError::AbsolutePathDisallowed(trimmed.to_string()));
                }
                Component::ParentDir => {
                    return Err(PolicyError::PathTraversal {
                        path: trimmed.to_string(),
                    });
                }
                Component::CurDir => continue,
                Component::Normal(c) => clean_components.push(c),
            }
        }

        if clean_components.is_empty() {
            if trimmed == "." {
                return Ok(workspace_root.to_path_buf());
            }
            return Err(PolicyError::InvalidPath(
                "Path resolves to empty".to_string(),
            ));
        }

        let mut resolved = workspace_root.to_path_buf();
        for c in clean_components {
            resolved.push(c);
        }

        Ok(resolved)
    }

    /// Path components as Windows resolves them: lowercased, `.` segments dropped and
    /// trailing dots/spaces trimmed, so `DATA/x`, `./data/x` and `data./x` compare equal.
    fn canonical_components(relative_path: &str) -> Vec<String> {
        relative_path
            .split(['/', '\\'])
            .map(|c| c.trim_end_matches(['.', ' ']).to_lowercase())
            .filter(|c| !c.is_empty())
            .collect()
    }

    /// Whether a path is inside `data/`, which holds the app database and agent
    /// definitions. Agents may never read or write it, whatever their permissions.
    pub fn is_app_data_path(relative_path: &str) -> bool {
        Self::canonical_components(relative_path)
            .first()
            .is_some_and(|c| c == "data")
    }

    /// Checks whether a relative path is a protected system directory or file
    /// that tools are strictly forbidden from modifying (e.g. agents/, config.toml, .versions/).
    pub fn is_protected_path(relative_path: &str) -> bool {
        let components = Self::canonical_components(relative_path);
        match components.as_slice() {
            [] => false,
            [only] if only == "config.toml" || only == ".env" => true,
            [first, ..] => matches!(first.as_str(), ".git" | "agents" | ".versions" | "data"),
        }
    }

    /// Resolve a path an agent wants to read, rejecting the app data directory.
    pub fn check_read_path(
        workspace_root: &Path,
        relative_path: &str,
    ) -> Result<PathBuf, PolicyError> {
        let resolved = Self::resolve_within_workspace(workspace_root, relative_path)?;
        if Self::is_app_data_path(relative_path) {
            return Err(PolicyError::PermissionDenied {
                agent_id: "system".to_string(),
                operation: "fs_read".to_string(),
                reason: format!("'{relative_path}' is inside the app data directory"),
            });
        }
        Ok(resolved)
    }

    /// Resolve and sanitize a relative path strictly within workspace_root,
    /// and verify that it is NOT a protected system path.
    pub fn check_write_path(
        workspace_root: &Path,
        relative_path: &str,
    ) -> Result<PathBuf, PolicyError> {
        let resolved = Self::resolve_within_workspace(workspace_root, relative_path)?;
        if Self::is_protected_path(relative_path) {
            return Err(PolicyError::PermissionDenied {
                agent_id: "system".to_string(),
                operation: "fs_write".to_string(),
                reason: format!(
                    "Writing to protected path '{relative_path}' is strictly forbidden"
                ),
            });
        }
        Ok(resolved)
    }
}
