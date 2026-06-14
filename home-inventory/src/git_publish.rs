use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

pub struct PublishResult {
    pub committed: bool,
    pub pushed: bool,
}

/// Stellt fest, ob es im git work tree (work_dir) Aenderungen an den uebergebenen
/// Pfaden gibt — sowohl unstaged als auch untracked. `git status --porcelain --`
/// liefert eine Zeile pro geaenderter Datei, leerer Output heisst sauber.
pub fn is_dirty(work_dir: &Path, paths: &[&Path]) -> Result<bool> {
    let mut args: Vec<String> = vec!["status".into(), "--porcelain".into(), "--".into()];
    for p in paths {
        args.push(p.display().to_string());
    }
    let out = git_output(work_dir, &args)?;
    Ok(!out.trim().is_empty())
}

/// Wenn paths einen Diff zeigen: git add, git commit (mit message), optional git push.
/// Wenn sauber: no-op und committed=false.
pub fn commit_and_push(
    work_dir: &Path,
    paths: &[&Path],
    message: &str,
    push: bool,
) -> Result<PublishResult> {
    if !is_dirty(work_dir, paths)? {
        return Ok(PublishResult {
            committed: false,
            pushed: false,
        });
    }

    let mut add_args: Vec<String> = vec!["add".into(), "--".into()];
    for p in paths {
        add_args.push(p.display().to_string());
    }
    git_run(work_dir, &add_args)?;

    git_run(
        work_dir,
        &["commit".into(), "-m".into(), message.to_string()],
    )?;

    let mut pushed = false;
    if push {
        git_run(work_dir, &["push".into()])?;
        pushed = true;
    }

    Ok(PublishResult {
        committed: true,
        pushed,
    })
}

fn git_run(work_dir: &Path, args: &[String]) -> Result<()> {
    let out = Command::new("git")
        .arg("-C")
        .arg(work_dir)
        .args(args.iter().map(|s| s.as_str()))
        .output()
        .with_context(|| format!("git {args:?} starten"))?;
    if !out.status.success() {
        bail!(
            "git {args:?} fehlgeschlagen ({}): {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(())
}

fn git_output(work_dir: &Path, args: &[String]) -> Result<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(work_dir)
        .args(args.iter().map(|s| s.as_str()))
        .output()
        .with_context(|| format!("git {args:?} starten"))?;
    if !out.status.success() {
        bail!(
            "git {args:?} fehlgeschlagen ({}): {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    String::from_utf8(out.stdout).context("git-Output ist kein UTF-8")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_repo(dir: &Path) {
        run(dir, &["init", "-q", "-b", "main"]);
        run(dir, &["config", "user.email", "test@example.com"]);
        run(dir, &["config", "user.name", "Test"]);
        std::fs::write(dir.join("README.md"), "initial\n").unwrap();
        run(dir, &["add", "README.md"]);
        run(dir, &["commit", "-q", "-m", "init"]);
    }

    fn run(dir: &Path, args: &[&str]) {
        let s = Command::new("git").arg("-C").arg(dir).args(args).status().unwrap();
        assert!(s.success(), "git {args:?}");
    }

    fn commit_count(dir: &Path) -> usize {
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["log", "--oneline"])
            .output()
            .unwrap();
        String::from_utf8(out.stdout).unwrap().lines().count()
    }

    #[test]
    fn no_change_no_commit() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(tmp.path());
        let before = commit_count(tmp.path());
        let r = commit_and_push(tmp.path(), &[Path::new("README.md")], "msg", false).unwrap();
        assert!(!r.committed);
        assert_eq!(commit_count(tmp.path()), before);
    }

    #[test]
    fn new_file_creates_commit() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(tmp.path());
        let yaml_dir = tmp.path().join("yaml");
        std::fs::create_dir(&yaml_dir).unwrap();
        let yaml_file = yaml_dir.join("ha.yaml");
        std::fs::write(&yaml_file, "- name: foo\n").unwrap();
        let before = commit_count(tmp.path());
        let r =
            commit_and_push(tmp.path(), &[&yaml_dir], "auto-sync ha", false).unwrap();
        assert!(r.committed);
        assert!(!r.pushed); // push=false
        assert_eq!(commit_count(tmp.path()), before + 1);
    }

    #[test]
    fn second_call_without_diff_is_noop() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(tmp.path());
        let yaml = tmp.path().join("ha.yaml");
        std::fs::write(&yaml, "v1\n").unwrap();
        let r1 = commit_and_push(tmp.path(), &[&yaml], "v1", false).unwrap();
        assert!(r1.committed);
        let before = commit_count(tmp.path());
        let r2 = commit_and_push(tmp.path(), &[&yaml], "v1", false).unwrap();
        assert!(!r2.committed);
        assert_eq!(commit_count(tmp.path()), before);
    }

    #[test]
    fn modification_creates_another_commit() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(tmp.path());
        let yaml = tmp.path().join("ha.yaml");
        std::fs::write(&yaml, "v1\n").unwrap();
        commit_and_push(tmp.path(), &[&yaml], "v1", false).unwrap();
        let before = commit_count(tmp.path());
        std::fs::write(&yaml, "v2\n").unwrap();
        let r = commit_and_push(tmp.path(), &[&yaml], "v2", false).unwrap();
        assert!(r.committed);
        assert_eq!(commit_count(tmp.path()), before + 1);
    }

    #[test]
    fn push_to_local_bare_remote_works() {
        let parent = tempfile::tempdir().unwrap();
        let bare = parent.path().join("bare.git");
        let work = parent.path().join("work");
        run(parent.path(), &["init", "-q", "--bare", bare.to_str().unwrap()]);
        std::fs::create_dir(&work).unwrap();
        init_repo(&work);
        run(&work, &["remote", "add", "origin", bare.to_str().unwrap()]);
        run(&work, &["push", "-q", "-u", "origin", "main"]);

        let yaml = work.join("ha.yaml");
        std::fs::write(&yaml, "v1\n").unwrap();
        let r = commit_and_push(&work, &[&yaml], "v1", true).unwrap();
        assert!(r.committed);
        assert!(r.pushed);

        // Im bare-Repo muss main HEAD jetzt zeigen
        let out = Command::new("git")
            .arg("-C")
            .arg(&bare)
            .args(["log", "--oneline", "main"])
            .output()
            .unwrap();
        let log = String::from_utf8(out.stdout).unwrap();
        assert!(log.contains("v1"), "bare-Repo zeigt v1-Commit nicht: {log}");
    }
}
