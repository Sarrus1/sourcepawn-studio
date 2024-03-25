use vfs::{AnchoredPath, FileId, FileSet, FileSetConfig, VfsPath};

/// Files are grouped into source roots. A source root is a directory on the
/// file systems which is watched for changes. Typically it corresponds to a
/// Rust crate. Source roots *might* be nested: in this case, a file belongs to
/// the nearest enclosing source root. Paths to files are always relative to a
/// source root, and the analyzer does not know the root path of the source root at
/// all. So, a file from one source root can't refer to a file in another source
/// root by path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourceRootId(pub u32);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceRoot {
    /// Include directory.
    ///
    /// Include directories are considered mostly immutable, this assumption is used to
    /// optimize salsa's query structure.
    pub is_include_dir: bool,
    file_set: FileSet,
    root: VfsPath,
}

impl SourceRoot {
    pub fn new_local(file_set: FileSet, root: VfsPath) -> SourceRoot {
        SourceRoot {
            is_include_dir: false,
            file_set,
            root,
        }
    }

    pub fn new_include_dir(file_set: FileSet, root: VfsPath) -> SourceRoot {
        SourceRoot {
            is_include_dir: true,
            file_set,
            root,
        }
    }

    pub fn path_for_file(&self, file: &FileId) -> Option<&VfsPath> {
        self.file_set.path_for_file(file)
    }

    pub fn file_for_path(&self, path: &VfsPath) -> Option<&FileId> {
        self.file_set.file_for_path(path)
    }

    pub fn resolve_path(&self, path: &AnchoredPath<'_>) -> Option<FileId> {
        self.file_set.resolve_path(*path)
    }

    pub fn resolve_path_relative_to_root(&self, path: &str) -> Option<FileId> {
        self.file_set
            .resolve_path_relative_to_root(&self.root, path)
    }

    pub fn iter(&self) -> impl Iterator<Item = FileId> + '_ {
        self.file_set.iter()
    }
}

#[derive(Default, Debug)]
pub struct SourceRootConfig {
    pub fsc: FileSetConfig,
}

impl SourceRootConfig {
    pub fn partition(&mut self, vfs: &vfs::Vfs) -> Vec<SourceRoot> {
        self.fsc
            .partition(vfs)
            .into_iter()
            .enumerate()
            // Assume that the first file set is the local one.
            .map(|(idx, (file_set, root))| {
                if idx == 0 {
                    SourceRoot::new_local(file_set, root)
                } else {
                    SourceRoot::new_include_dir(file_set, root)
                }
            })
            .collect()
    }
}
