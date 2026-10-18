//! Match merged project settings against ordered native path spellings.

use crate::config_toml::ConfigToml;
use crate::config_toml::ProjectConfig;
use codex_utils_path::normalize_for_path_comparison;
use codex_utils_path_uri::LegacyAppPathString;
use codex_utils_path_uri::PathConvention;
use std::path::Path;

/// Original and executor-normalized canonical spellings of one native path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectTrustPath {
    pub original: String,
    pub canonical: Option<String>,
}

/// Ordered project lookup keys for one executor working directory.
///
/// Resolve this independently of configuration layers, then apply it after all
/// local and cloud projects maps have been merged. Cwd entries, including an
/// entry without a trust level, take precedence over repository-root entries.
/// If no exact entry matches, inherit from the nearest configured ancestor of
/// the canonical cwd (or its original spelling when canonicalization fails).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectTrustLookup {
    convention: PathConvention,
    keys: Vec<String>,
}

impl ProjectTrustLookup {
    pub(crate) fn from_native_paths(path: &Path, repo_root: Option<&Path>) -> Self {
        let trust_path = |path: &Path| ProjectTrustPath {
            original: path.to_string_lossy().into_owned(),
            canonical: normalize_for_path_comparison(path)
                .ok()
                .map(|path| path.to_string_lossy().into_owned()),
        };
        Self::from_paths(
            PathConvention::native(),
            trust_path(path),
            repo_root.map(trust_path),
        )
    }

    /// Build lookup keys for a cwd and optional repository root.
    ///
    /// Cwd keys always take precedence over repository-root keys. Each
    /// canonical spelling is tried before its original spelling; `None` uses
    /// only the original. This method performs no filesystem access or path
    /// resolution. Callers must supply executor-normalized canonical spellings,
    /// including Windows extended-length prefixes and WSL mounted-drive folding
    /// where applicable. Original spellings remain literal. Windows keys are
    /// matched without ASCII case distinctions; POSIX keys remain case-sensitive.
    pub fn from_paths(
        convention: PathConvention,
        cwd: ProjectTrustPath,
        repo_root: Option<ProjectTrustPath>,
    ) -> Self {
        let mut keys = Vec::new();
        for ProjectTrustPath {
            original,
            canonical,
        } in std::iter::once(cwd).chain(repo_root)
        {
            let original = normalize_lookup_key(&original, convention);
            let canonical = canonical
                .map(|key| normalize_lookup_key(&key, convention))
                .unwrap_or_else(|| original.clone());
            if canonical != original {
                keys.push(canonical);
            }
            keys.push(original);
        }
        Self { convention, keys }
    }
}

impl ConfigToml {
    /// Select the active project from the final merged projects map.
    pub fn get_active_project_for_lookup(
        &self,
        lookup: &ProjectTrustLookup,
    ) -> Option<ProjectConfig> {
        let projects = self.projects.as_ref()?;
        for key in &lookup.keys {
            if let Some(project) = projects.get(key) {
                return Some(project.clone());
            }
            // Preserve native exact-key preference and deterministic selection
            // when multiple Windows keys differ only in ASCII case.
            if let Some((_, project)) = projects
                .iter()
                .filter(|(candidate, _)| normalize_lookup_key(candidate, lookup.convention) == *key)
                .min_by_key(|(candidate, _)| *candidate)
            {
                return Some(project.clone());
            }
        }
        // The first key is canonical: do not inherit from a symlink's original parent.
        let key = ancestor_project_key(lookup.keys.first()?, lookup.convention, projects.keys())?;
        projects.get(key).cloned()
    }
}

fn normalize_lookup_key(key: &str, convention: PathConvention) -> String {
    match convention {
        PathConvention::Windows => key.to_ascii_lowercase(),
        PathConvention::Posix => key.to_owned(),
    }
}

/// Find the nearest configured strict ancestor using executor path semantics.
/// Call only after exact cwd/project/repository decisions have been exhausted.
pub(crate) fn ancestor_project_key<'a>(
    path: &str,
    convention: PathConvention,
    keys: impl IntoIterator<Item = &'a String>,
) -> Option<&'a String> {
    let parse = |key: &str| {
        let key = normalize_lookup_key(key, convention);
        // Lookup keys lowercase UNC namespaces, but the URI parser expects `UNC`.
        let key = if convention == PathConvention::Windows {
            key.strip_prefix(r"\\?\unc\")
                .or_else(|| key.strip_prefix(r"\\.\unc\"))
                .map(|unc| format!(r"\\{unc}"))
                .unwrap_or(key)
        } else {
            key
        };
        LegacyAppPathString::from_string(key)
            .to_path_uri(convention)
            .ok()
    };
    let mut parent = parse(path)?.parent();
    let candidates: Vec<_> = keys
        .into_iter()
        .filter_map(|key| Some((key, parse(key)?)))
        .collect();
    while let Some(dir) = parent {
        let normalized = normalize_lookup_key(&dir.inferred_native_path_string(), convention);
        if let Some((key, _)) = candidates
            .iter()
            // URI equality retains trailing separators, unlike native ancestry.
            .filter(|(_, candidate)| candidate.relative_path_from(&dir).as_deref() == Some(""))
            .min_by_key(|(key, _)| (key.as_str() != normalized, *key))
        {
            return Some(key);
        }
        parent = dir.parent();
    }
    None
}

#[cfg(test)]
#[path = "project_trust_tests.rs"]
mod tests;
