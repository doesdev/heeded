//! `heeded update`: the latest release in place of this build, then the
//! plugin. Downloads go through curl, the way git calls go through git.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::consts::{claude_cmd, releases, releases_api};
use crate::sha256::sha256;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const EXE: &str = std::env::consts::EXE_SUFFIX;
const PLUGIN_STEPS: [&[&str]; 2] = [
    &["plugin", "marketplace", "update", "heeded"],
    &["plugin", "update", "heeded@heeded"],
];

/// The release build for this machine, by its Rust target name.
pub fn target() -> Option<&'static str> {
    let builds = [
        ("windows", "x86_64", "x86_64-pc-windows-msvc"),
        ("linux", "x86_64", "x86_64-unknown-linux-musl"),
        ("macos", "aarch64", "aarch64-apple-darwin"),
        ("macos", "x86_64", "x86_64-apple-darwin"),
    ];

    builds
        .into_iter()
        .find(|(os, arch, _)| *os == std::env::consts::OS && *arch == std::env::consts::ARCH)
        .map(|(_, _, target)| target)
}

pub fn asset_for(tag: &str, target: &str) -> String {
    format!("heeded-{tag}-{target}{EXE}")
}

fn triple(version: &str) -> Option<(u64, u64, u64)> {
    let mut parts = version.trim_start_matches('v').split('.');
    let mut next = || parts.next()?.parse::<u64>().ok();
    let found = (next()?, next()?, next()?);

    parts.next().is_none().then_some(found)
}

/// Is the release tagged `tag` newer than `version`. `None` when either is
/// not `X.Y.Z`.
pub fn newer(tag: &str, version: &str) -> Option<bool> {
    Some(triple(tag)? > triple(version)?)
}

/// The digest `SHA256SUMS` gives for `name`.
pub fn sum_for(sums: &str, name: &str) -> Option<String> {
    sums.lines().find_map(|line| {
        let (digest, file) = line.trim_end().split_once(char::is_whitespace)?;
        let file = file.trim_start().trim_start_matches('*');

        (file == name).then(|| digest.to_ascii_lowercase())
    })
}

fn curl(url: &str, out: Option<&Path>) -> Result<Vec<u8>, String> {
    let mut command = Command::new("curl");
    command.args(["-fsSL", url]);
    if let Some(out) = out {
        command.arg("-o").arg(out);
    }

    let ran = match command.output() {
        Ok(ran) => ran,
        Err(ex) if ex.kind() == std::io::ErrorKind::NotFound => {
            return Err("heeded update needs curl, and it is not on the PATH.".to_string());
        }
        Err(ex) => return Err(format!("curl: {ex}")),
    };
    if !ran.status.success() {
        return Err(format!(
            "Could not reach {url}: {}",
            String::from_utf8_lossy(&ran.stderr).trim()
        ));
    }

    Ok(ran.stdout)
}

fn latest() -> Result<String, String> {
    let api = releases_api();
    let body = curl(&api, None)?;

    serde_json::from_slice::<serde_json::Value>(&body)
        .ok()
        .and_then(|release| release.get("tag_name")?.as_str().map(str::to_string))
        .ok_or_else(|| format!("{api} did not name a release."))
}

fn runs(file: &Path) -> Result<String, String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(file, std::fs::Permissions::from_mode(0o755))
            .map_err(|ex| format!("{}: {ex}", file.display()))?;
    }

    let ran = Command::new(file)
        .arg("--version")
        .output()
        .map_err(|ex| format!("The downloaded build does not run here: {ex}"))?;
    let said = String::from_utf8_lossy(&ran.stdout).trim().to_string();

    if !ran.status.success() || !said.starts_with("heeded ") {
        return Err(format!(
            "The downloaded build does not run here: it said {said:?}"
        ));
    }

    Ok(said)
}

fn aside(exe: &Path) -> PathBuf {
    exe.with_file_name(format!("heeded.old{EXE}"))
}

/// Puts `fresh` where `exe` is. A running file cannot be overwritten on
/// Windows but can be renamed, so the old one is moved aside first and
/// put back if the new one cannot take its place.
pub fn swap(exe: &Path, fresh: &Path) -> Result<(), String> {
    let fail = |ex: std::io::Error| format!("{}: {ex}", exe.display());

    if !cfg!(windows) {
        return std::fs::rename(fresh, exe).map_err(fail);
    }

    let old = aside(exe);
    let _ = std::fs::remove_file(&old);
    std::fs::rename(exe, &old).map_err(fail)?;
    if let Err(ex) = std::fs::rename(fresh, exe) {
        let _ = std::fs::rename(&old, exe);
        return Err(fail(ex));
    }

    Ok(())
}

fn install(exe: &Path, tag: &str, target: &str) -> Result<String, String> {
    let name = asset_for(tag, target);
    let base = format!("{}/{tag}", releases().trim_end_matches('/'));
    let sums = curl(&format!("{base}/SHA256SUMS"), None)?;
    let wanted = sum_for(&String::from_utf8_lossy(&sums), &name)
        .ok_or_else(|| format!("SHA256SUMS for {tag} has no line for {name}."))?;
    let fresh = exe.with_file_name(format!("heeded.download{EXE}"));
    let _ = std::fs::remove_file(&fresh);

    let checked = curl(&format!("{base}/{name}"), Some(&fresh)).and_then(|_| {
        let data = std::fs::read(&fresh).map_err(|ex| format!("{}: {ex}", fresh.display()))?;
        let got = sha256(&data);
        if got != wanted {
            return Err(format!(
                "{name} does not match SHA256SUMS ({got}, not {wanted}). Nothing was changed."
            ));
        }
        runs(&fresh)
    });
    let said = match checked {
        Ok(said) => said,
        Err(ex) => {
            let _ = std::fs::remove_file(&fresh);
            return Err(ex);
        }
    };

    swap(exe, &fresh)?;

    Ok(said)
}

fn plugin(tag: &str) {
    let claude = claude_cmd();
    let by_hand = |from: usize| {
        let steps: Vec<String> = PLUGIN_STEPS[from..]
            .iter()
            .map(|step| format!("  claude {}", step.join(" ")))
            .collect();
        steps.join("\n")
    };

    for (at, step) in PLUGIN_STEPS.iter().enumerate() {
        match Command::new(&claude).args(*step).output() {
            Ok(ran) if ran.status.success() => println!("Ran: claude {}", step.join(" ")),
            Ok(_) => {
                println!(
                    "claude {} failed. Run it yourself, and what follows:\n{}",
                    step.join(" "),
                    by_hand(at)
                );
                return;
            }
            Err(_) => {
                println!(
                    "Claude Code was not found, so update the plugin yourself:\n{}",
                    by_hand(at)
                );
                return;
            }
        }
    }

    println!(
        "A marketplace added from a folder updates from that folder, so that \
checkout must be at {tag} too."
    );
}

/// Finds the latest release and, unless `check`, installs it when newer.
pub fn update(check: bool) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|ex| format!("Where is heeded? {ex}"))?;
    let _ = std::fs::remove_file(aside(&exe));
    let tag = latest()?;
    let Some(ahead) = newer(&tag, VERSION) else {
        return Err(format!("The latest release is tagged {tag}, not vX.Y.Z."));
    };

    if !ahead {
        println!("heeded {VERSION} is up to date (the latest release is {tag}).");
        return Ok(());
    }
    let Some(target) = target() else {
        return Err(format!(
            "{tag} is out, but there is no release build for {} {}. Build it with cargo install.",
            std::env::consts::OS,
            std::env::consts::ARCH
        ));
    };
    if check {
        println!("heeded {VERSION} is installed and {tag} is out. heeded update installs it.");
        return Ok(());
    }

    let said = install(&exe, &tag, target)?;
    println!("Updated {} to {tag} (it says {said}).", exe.display());
    plugin(&tag);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_only_moves_forward() {
        let cases = [
            ("v0.15.0", "0.14.0", Some(true)),
            ("v1.0.0", "0.14.9", Some(true)),
            ("v0.14.10", "0.14.9", Some(true)),
            ("v0.14.0", "0.14.0", Some(false)),
            ("v0.13.0", "0.14.0", Some(false)),
            ("0.15.0", "0.14.0", Some(true)),
            ("v0.15", "0.14.0", None),
            ("v0.15.0-rc1", "0.14.0", None),
            ("v0.15.0.1", "0.14.0", None),
            ("latest", "0.14.0", None),
        ];

        for (tag, version, is) in cases {
            assert_eq!(newer(tag, version), is, "{tag} over {version}");
        }
    }

    #[test]
    fn sum_for_reads_sha256sum_lines() {
        let sums = "AB12  heeded-v1.0.0-x86_64-apple-darwin\n\
cd34 *heeded-v1.0.0-x86_64-pc-windows-msvc.exe\r\n\
ef56  heeded-v1.0.0-x86_64-unknown-linux-musl\n";
        let cases = [
            ("heeded-v1.0.0-x86_64-apple-darwin", Some("ab12")),
            ("heeded-v1.0.0-x86_64-pc-windows-msvc.exe", Some("cd34")),
            ("heeded-v1.0.0-x86_64-unknown-linux-musl", Some("ef56")),
            ("heeded-v1.0.0-aarch64-apple-darwin", None),
            ("x86_64-apple-darwin", None),
        ];

        for (name, digest) in cases {
            assert_eq!(sum_for(sums, name).as_deref(), digest, "{name}");
        }
    }

    #[test]
    fn asset_names_match_the_release_workflow() {
        let exe = if cfg!(windows) { ".exe" } else { "" };

        assert_eq!(
            asset_for("v1.2.3", "x86_64-pc-windows-msvc"),
            format!("heeded-v1.2.3-x86_64-pc-windows-msvc{exe}")
        );
    }
}
