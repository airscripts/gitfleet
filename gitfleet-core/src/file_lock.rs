use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;

pub(crate) struct FileLock {
    file: File,
}

impl FileLock {
    pub(crate) fn shared(path: &Path) -> io::Result<Self> {
        Self::open(path, false)
    }

    pub(crate) fn exclusive(path: &Path) -> io::Result<Self> {
        Self::open(path, true)
    }

    fn open(path: &Path, exclusive: bool) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
        }

        if exclusive {
            file.lock()?;
        } else {
            file.lock_shared()?;
        }

        Ok(Self { file })
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_and_exclusive_locks_can_be_reacquired() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("state.lock");

        {
            let _lock = FileLock::shared(&path).unwrap();
        }

        {
            let _lock = FileLock::exclusive(&path).unwrap();
        }

        assert!(path.exists());
    }
}
