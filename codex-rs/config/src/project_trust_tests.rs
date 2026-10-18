//! Exercises project-key precedence and map matching across path conventions.

use super::*;
use codex_protocol::config_types::TrustLevel;
use pretty_assertions::assert_eq;

fn config_with_projects(entries: &[(&str, Option<TrustLevel>)]) -> ConfigToml {
    ConfigToml {
        projects: Some(
            entries
                .iter()
                .map(|(key, trust_level)| {
                    (
                        key.to_string(),
                        ProjectConfig {
                            trust_level: *trust_level,
                        },
                    )
                })
                .collect(),
        ),
        ..Default::default()
    }
}

fn assert_lookup_order(lookup: &ProjectTrustLookup, entries: &[(&str, Option<TrustLevel>)]) {
    let mut config = config_with_projects(entries);
    for (key, trust_level) in entries {
        assert_eq!(
            config.get_active_project_for_lookup(lookup),
            Some(ProjectConfig {
                trust_level: *trust_level
            }),
            "expected {key} for {lookup:?}",
        );
        config.projects.as_mut().unwrap().remove(*key);
    }
    assert_eq!(config.get_active_project_for_lookup(lookup), None);
    assert_eq!(
        ConfigToml::default().get_active_project_for_lookup(lookup),
        None
    );
}

#[test]
fn posix_lookup_preserves_cwd_root_alias_order_and_case() {
    let lookup = ProjectTrustLookup::from_paths(
        PathConvention::Posix,
        ProjectTrustPath {
            original: "/alias/src".into(),
            canonical: Some("/repo/src".into()),
        },
        Some(ProjectTrustPath {
            original: "/alias".into(),
            canonical: Some("/repo".into()),
        }),
    );
    assert_lookup_order(
        &lookup,
        &[
            ("/repo/src", None),
            ("/alias/src", Some(TrustLevel::Trusted)),
            ("/repo", Some(TrustLevel::Untrusted)),
            ("/alias", Some(TrustLevel::Trusted)),
        ],
    );
    assert_eq!(
        config_with_projects(&[("/Repo/src", Some(TrustLevel::Trusted))])
            .get_active_project_for_lookup(&lookup),
        None,
    );
}

#[test]
fn windows_prefers_exact_normalized_key_then_sorted_aliases() {
    let lookup = ProjectTrustLookup::from_paths(
        PathConvention::Windows,
        ProjectTrustPath {
            original: r"C:\Repo".into(),
            canonical: None,
        },
        /*repo_root*/ None,
    );
    assert_lookup_order(
        &lookup,
        &[
            (r"c:\repo", Some(TrustLevel::Trusted)),
            (r"C:\REPO", Some(TrustLevel::Untrusted)),
            (r"C:\Repo", Some(TrustLevel::Trusted)),
        ],
    );
}

#[test]
fn ancestor_trust_is_nearest_and_respects_path_boundaries() {
    let lookup = ProjectTrustLookup::from_paths(
        PathConvention::Posix,
        ProjectTrustPath {
            original: "/work/repos/project/src".into(),
            canonical: None,
        },
        None,
    );
    assert_lookup_order(
        &lookup,
        &[
            ("/work/repos/project", Some(TrustLevel::Untrusted)),
            ("/work/repos/", Some(TrustLevel::Trusted)),
            ("/work", Some(TrustLevel::Untrusted)),
            ("/", Some(TrustLevel::Trusted)),
        ],
    );
    for key in [
        "/work/repo",
        "/Work/repos",
        "work/repos",
        "/work/repos/project/src/child",
    ] {
        assert_eq!(
            config_with_projects(&[(key, Some(TrustLevel::Trusted))])
                .get_active_project_for_lookup(&lookup),
            None,
            "must not inherit from {key}",
        );
    }
}

#[test]
fn exact_repository_decisions_precede_ancestor_fallback() {
    let lookup = ProjectTrustLookup::from_paths(
        PathConvention::Posix,
        ProjectTrustPath {
            original: "/work/project/src".into(),
            canonical: None,
        },
        Some(ProjectTrustPath {
            original: "/work/project".into(),
            canonical: None,
        }),
    );
    assert_lookup_order(
        &lookup,
        &[
            ("/work/project/src", None),
            ("/work/project", Some(TrustLevel::Untrusted)),
            ("/work", Some(TrustLevel::Trusted)),
        ],
    );
}

#[test]
fn ancestor_fallback_uses_canonical_path_not_symlink_parent() {
    let lookup = ProjectTrustLookup::from_paths(
        PathConvention::Posix,
        ProjectTrustPath {
            original: "/trusted/link/project".into(),
            canonical: Some("/outside/project".into()),
        },
        None,
    );
    let config = config_with_projects(&[("/trusted", Some(TrustLevel::Trusted))]);
    assert_eq!(config.get_active_project_for_lookup(&lookup), None);
    assert_lookup_order(
        &lookup,
        &[
            ("/trusted/link/project", Some(TrustLevel::Trusted)),
            ("/outside", Some(TrustLevel::Untrusted)),
        ],
    );
}

#[test]
fn windows_ancestor_trust_handles_drives_unc_and_extended_paths() {
    for (cwd, parent, outside) in [
        (r"C:\Work\Repo\src", r"c:\work", r"C:\Wor"),
        (r"\\?\C:\Work\Repo", r"C:\WORK", r"D:\Work"),
        (r"\\server\share\Repo", r"\\SERVER\share", r"\\server\other"),
        (r"\\?\UNC\server\share\Repo", r"\\server\share", r"\\server"),
    ] {
        let lookup = ProjectTrustLookup::from_paths(
            PathConvention::Windows,
            ProjectTrustPath {
                original: cwd.into(),
                canonical: None,
            },
            None,
        );
        assert_lookup_order(&lookup, &[(parent, Some(TrustLevel::Trusted))]);
        assert_eq!(
            config_with_projects(&[(outside, Some(TrustLevel::Trusted))])
                .get_active_project_for_lookup(&lookup),
            None,
        );
    }
}

#[cfg(unix)]
#[test]
fn native_ancestor_trust_resolves_symlinks_and_preserves_repo_denials() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let trusted = temp.path().join("trusted");
    let outside = temp.path().join("outside");
    let repo = trusted.join("repo");
    std::fs::create_dir_all(repo.join("src"))?;
    std::fs::create_dir_all(&outside)?;
    std::os::unix::fs::symlink(&outside, trusted.join("escape"))?;
    std::os::unix::fs::symlink(&repo, temp.path().join("alias"))?;
    let parent_key = trusted.to_str().unwrap();
    let repo_key = repo.to_str().unwrap();
    let config = config_with_projects(&[(parent_key, Some(TrustLevel::Trusted))]);
    assert_eq!(
        config
            .get_active_project(&repo.join("src"), Some(&repo))
            .unwrap()
            .trust_level,
        Some(TrustLevel::Trusted)
    );
    assert_eq!(
        config
            .get_active_project(&temp.path().join("alias/src"), None)
            .unwrap()
            .trust_level,
        Some(TrustLevel::Trusted)
    );
    assert_eq!(
        config.get_active_project(&trusted.join("escape"), None),
        None
    );
    let config = config_with_projects(&[
        (parent_key, Some(TrustLevel::Trusted)),
        (repo_key, Some(TrustLevel::Untrusted)),
    ]);
    assert_eq!(
        config
            .get_active_project(&repo.join("src"), Some(&repo))
            .unwrap()
            .trust_level,
        Some(TrustLevel::Untrusted)
    );
    Ok(())
}
