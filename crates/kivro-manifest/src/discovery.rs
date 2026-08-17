//! unward search for a project manifest

use std::path::{Path, PathBuf};

use kivro_core::{Error, Result};

use crate::MANIFEST_FILENAME;

/// walk up from start dir looking for manifest file
pub fn discover_from(start: &Path) -> Result<PathBuf> {
    let start = if start.is_absolute() {
        start.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| Error::io("resolve current directory", start, e))?
            .join(start)
    };

    let mut dir: Option<&Path> = Some(start.as_path());
    while let Some(current) = dir {
        let candidate = current.join(MANIFEST_FILENAME);
        if candidate.is_file() {
            return Ok(candidate);
        }
        dir = current.parent();
    }
    Err(Error::ManifestNotFound {
        filename: MANIFEST_FILENAME,
        start,
    })
}

/// walk up from current working dir
pub fn discover() -> Result<PathBuf> {
    let cwd = std::env::current_dir().map_err(Error::RawIo)?;
    discover_from(&cwd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_manifest_in_ancestor_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::write(root.join(MANIFEST_FILENAME), "[project]\nname = \"x\"\n").unwrap();
        let nested = root.join("src").join("deep").join("deeper");
        std::fs::create_dir_all(&nested).unwrap();

        let found = discover_from(&nested).unwrap();
        assert_eq!(found, root.join(MANIFEST_FILENAME));
    }

    #[test]
    fn prefers_the_nearest_manifest() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let inner = root.join("packages").join("app");
        std::fs::create_dir_all(&inner).unwrap();
        std::fs::write(
            root.join(MANIFEST_FILENAME),
            "[project]\nname = \"outer\"\n",
        )
        .unwrap();
        std::fs::write(
            inner.join(MANIFEST_FILENAME),
            "[project]\nname = \"inner\"\n",
        )
        .unwrap();

        assert_eq!(
            discover_from(&inner).unwrap(),
            inner.join(MANIFEST_FILENAME)
        );
    }

    #[test]
    fn reports_a_helpful_error_when_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let err = discover_from(tmp.path()).unwrap_err();
        assert_eq!(err.kind(), "manifest_not_found");
        assert!(err.hint().unwrap().contains("kivro init"));
    }
}
