use std::ops::Deref;
use std::path::Path;

use camino::{Utf8Path, Utf8PathBuf};

use crate::file_system::{FileSystemPath, FileSystemPathBuf};

#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct VendoredPath(Utf8Path);

impl VendoredPath {
    pub fn new(path: &(impl AsRef<Utf8Path> + ?Sized)) -> &Self {
        let path = path.as_ref();
        // SAFETY: VendoredPath is marked as #[repr(transparent)] so the conversion from a
        // *const Utf8Path to a *const VendoredPath is valid.
        unsafe { &*(path as *const Utf8Path as *const VendoredPath) }
    }

    pub fn to_path_buf(&self) -> VendoredPathBuf {
        VendoredPathBuf(self.0.to_path_buf())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct VendoredPathBuf(Utf8PathBuf);

impl Default for VendoredPathBuf {
    fn default() -> Self {
        Self::new()
    }
}

impl VendoredPathBuf {
    pub fn new() -> Self {
        Self(Utf8PathBuf::new())
    }

    pub fn as_path(&self) -> &VendoredPath {
        VendoredPath::new(&self.0)
    }
}

impl AsRef<VendoredPath> for VendoredPathBuf {
    fn as_ref(&self) -> &VendoredPath {
        self.as_path()
    }
}

impl AsRef<VendoredPath> for VendoredPath {
    #[inline]
    fn as_ref(&self) -> &VendoredPath {
        self
    }
}

impl AsRef<VendoredPath> for str {
    #[inline]
    fn as_ref(&self) -> &VendoredPath {
        VendoredPath::new(self)
    }
}

impl AsRef<VendoredPath> for String {
    #[inline]
    fn as_ref(&self) -> &VendoredPath {
        VendoredPath::new(self)
    }
}

impl AsRef<Path> for VendoredPath {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.0.as_std_path()
    }
}

impl Deref for VendoredPathBuf {
    type Target = VendoredPath;

    fn deref(&self) -> &Self::Target {
        self.as_path()
    }
}

/// Path to a file.
///
/// The path abstracts that files in Ruff can come from different sources:
///
/// * a file stored on disk
/// * a vendored file that ships as part of the ruff binary
/// * Future: A virtual file that references a slice of another file. For example, the CSS code in a python file.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum VfsPath {
    /// Path that points to a file on disk.
    FileSystem(FileSystemPathBuf),
    Vendored(VendoredPathBuf),
}

impl VfsPath {
    /// Create a new path to a file on the file system.
    #[must_use]
    pub fn file_system(path: impl AsRef<FileSystemPath>) -> Self {
        VfsPath::FileSystem(path.as_ref().to_path_buf())
    }

    /// Returns `Some` if the path is a file system path that points to a path on disk.
    #[must_use]
    #[inline]
    pub fn into_file_system_path_buf(self) -> Option<FileSystemPathBuf> {
        match self {
            VfsPath::FileSystem(path) => Some(path),
            VfsPath::Vendored(_) => None,
        }
    }

    #[must_use]
    #[inline]
    pub fn as_file_system_path(&self) -> Option<&FileSystemPath> {
        match self {
            VfsPath::FileSystem(path) => Some(path.as_path()),
            VfsPath::Vendored(_) => None,
        }
    }

    /// Returns `true` if the path is a file system path that points to a path on disk.
    #[must_use]
    #[inline]
    pub const fn is_file_system_path(&self) -> bool {
        matches!(self, VfsPath::FileSystem(_))
    }

    /// Returns `true` if the path is a vendored path.
    #[must_use]
    #[inline]
    pub const fn is_vendored_path(&self) -> bool {
        matches!(self, VfsPath::Vendored(_))
    }

    #[must_use]
    #[inline]
    pub fn as_vendored_path(&self) -> Option<&VendoredPath> {
        match self {
            VfsPath::Vendored(path) => Some(path.as_path()),
            VfsPath::FileSystem(_) => None,
        }
    }

    /// Yields the underlying [`str`] slice.
    pub fn as_str(&self) -> &str {
        match self {
            VfsPath::FileSystem(path) => path.as_str(),
            VfsPath::Vendored(path) => path.as_str(),
        }
    }
}

impl AsRef<str> for VfsPath {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<FileSystemPathBuf> for VfsPath {
    fn from(value: FileSystemPathBuf) -> Self {
        Self::FileSystem(value)
    }
}

impl From<&FileSystemPath> for VfsPath {
    fn from(value: &FileSystemPath) -> Self {
        VfsPath::FileSystem(value.to_path_buf())
    }
}

impl From<VendoredPathBuf> for VfsPath {
    fn from(value: VendoredPathBuf) -> Self {
        Self::Vendored(value)
    }
}

impl From<&VendoredPath> for VfsPath {
    fn from(value: &VendoredPath) -> Self {
        Self::Vendored(value.to_path_buf())
    }
}

impl PartialEq<FileSystemPath> for VfsPath {
    #[inline]
    fn eq(&self, other: &FileSystemPath) -> bool {
        self.as_file_system_path()
            .is_some_and(|self_path| self_path == other)
    }
}

impl PartialEq<VfsPath> for FileSystemPath {
    #[inline]
    fn eq(&self, other: &VfsPath) -> bool {
        other == self
    }
}

impl PartialEq<FileSystemPathBuf> for VfsPath {
    #[inline]
    fn eq(&self, other: &FileSystemPathBuf) -> bool {
        self == other.as_path()
    }
}

impl PartialEq<VfsPath> for FileSystemPathBuf {
    fn eq(&self, other: &VfsPath) -> bool {
        other == self
    }
}

impl PartialEq<VendoredPath> for VfsPath {
    #[inline]
    fn eq(&self, other: &VendoredPath) -> bool {
        self.as_vendored_path()
            .is_some_and(|self_path| self_path == other)
    }
}

impl PartialEq<VfsPath> for VendoredPath {
    #[inline]
    fn eq(&self, other: &VfsPath) -> bool {
        other == self
    }
}

impl PartialEq<VendoredPathBuf> for VfsPath {
    #[inline]
    fn eq(&self, other: &VendoredPathBuf) -> bool {
        other.as_path() == self
    }
}

impl PartialEq<VfsPath> for VendoredPathBuf {
    #[inline]
    fn eq(&self, other: &VfsPath) -> bool {
        other == self
    }
}
