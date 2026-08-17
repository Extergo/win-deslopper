//! Shared cross-process exclusion for every real Owner Mode mutation family.

use std::{fs::File, path::PathBuf};

#[derive(Debug)]
pub(crate) struct OwnerMutationProcessLock {
    _file: File,
    #[cfg(not(windows))]
    path: PathBuf,
}

impl OwnerMutationProcessLock {
    pub(crate) fn acquire(path: PathBuf) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        #[cfg(windows)]
        {
            use std::{fs::OpenOptions, os::windows::fs::OpenOptionsExt};

            // The file is a stable rendezvous point, not ownership metadata. Windows owns the
            // lifetime through this exclusive handle; an existing but unopened file is stale-safe.
            let file = OpenOptions::new()
                .create(true)
                .truncate(false)
                .read(true)
                .write(true)
                .share_mode(0)
                .open(&path)
                .map_err(|_| "Another Deslopper change is still in progress.".to_owned())?;
            Ok(Self { _file: file })
        }
        #[cfg(not(windows))]
        {
            use std::fs::OpenOptions;

            let file = OpenOptions::new()
                .create_new(true)
                .read(true)
                .write(true)
                .open(&path)
                .map_err(|_| "Another Deslopper change is still in progress.".to_owned())?;
            Ok(Self { _file: file, path })
        }
    }
}

#[cfg(not(windows))]
impl Drop for OwnerMutationProcessLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHILD_LOCK_PATH: &str = "DESLOPPER_TEST_CHILD_LOCK_PATH";
    const CHILD_READY_PATH: &str = "DESLOPPER_TEST_CHILD_READY_PATH";
    const CHILD_RELEASE_PATH: &str = "DESLOPPER_TEST_CHILD_RELEASE_PATH";

    fn path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "deslopper-shared-owner-lock-{label}-{}.lock",
            std::process::id()
        ))
    }

    #[test]
    fn separate_live_owner_is_blocked_and_completed_owner_releases() {
        let path = path("exclusive");
        let _ = std::fs::remove_file(&path);
        let first = OwnerMutationProcessLock::acquire(path.clone()).unwrap();
        assert_eq!(
            OwnerMutationProcessLock::acquire(path.clone()).unwrap_err(),
            "Another Deslopper change is still in progress."
        );
        drop(first);
        let second = OwnerMutationProcessLock::acquire(path.clone()).unwrap();
        drop(second);
        let _ = std::fs::remove_file(path);
    }

    #[cfg(windows)]
    #[test]
    fn stale_lock_file_is_not_mistaken_for_live_ownership() {
        let path = path("stale");
        std::fs::write(&path, b"historical rendezvous file").unwrap();
        let owner = OwnerMutationProcessLock::acquire(path.clone()).unwrap();
        drop(owner);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn child_holds_owner_lock_when_requested() {
        let (Ok(lock_path), Ok(ready_path), Ok(release_path)) = (
            std::env::var(CHILD_LOCK_PATH),
            std::env::var(CHILD_READY_PATH),
            std::env::var(CHILD_RELEASE_PATH),
        ) else {
            return;
        };
        let owner = OwnerMutationProcessLock::acquire(PathBuf::from(lock_path)).unwrap();
        std::fs::write(&ready_path, b"ready").unwrap();
        for _ in 0..500 {
            if std::path::Path::new(&release_path).exists() {
                drop(owner);
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        panic!("parent did not release child lock test");
    }

    #[test]
    fn separate_process_cannot_hold_the_same_owner_lock() {
        let lock_path = path("child-process");
        let ready_path = lock_path.with_extension("ready");
        let release_path = lock_path.with_extension("release");
        for candidate in [&lock_path, &ready_path, &release_path] {
            let _ = std::fs::remove_file(candidate);
        }
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "mutation::process_lock::tests::child_holds_owner_lock_when_requested",
                "--nocapture",
            ])
            .env(CHILD_LOCK_PATH, &lock_path)
            .env(CHILD_READY_PATH, &ready_path)
            .env(CHILD_RELEASE_PATH, &release_path)
            .spawn()
            .unwrap();
        let mut ready = false;
        for _ in 0..500 {
            if ready_path.exists() {
                ready = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(ready, "child process never acquired the owner lock");
        assert_eq!(
            OwnerMutationProcessLock::acquire(lock_path.clone()).unwrap_err(),
            "Another Deslopper change is still in progress."
        );
        std::fs::write(&release_path, b"release").unwrap();
        assert!(child.wait().unwrap().success());
        let owner = OwnerMutationProcessLock::acquire(lock_path.clone()).unwrap();
        drop(owner);
        for candidate in [&lock_path, &ready_path, &release_path] {
            let _ = std::fs::remove_file(candidate);
        }
    }
}
