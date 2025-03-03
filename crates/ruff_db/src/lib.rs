use std::hash::BuildHasherDefault;

use rustc_hash::FxHasher;
use salsa::DbWithJar;

use crate::file_system::FileSystem;
use crate::parsed::parsed_module;
use crate::source::{line_index, source_text};
use crate::vfs::{Vfs, VfsFile};

pub mod file_system;
pub mod parsed;
pub mod source;
pub mod vfs;

pub(crate) type FxDashMap<K, V> = dashmap::DashMap<K, V, BuildHasherDefault<FxHasher>>;

#[salsa::jar(db=Db)]
pub struct Jar(VfsFile, source_text, line_index, parsed_module);

/// Database that gives access to the virtual filesystem, source code, and parsed AST.
pub trait Db: DbWithJar<Jar> {
    fn file_system(&self) -> &dyn FileSystem;

    fn vfs(&self) -> &Vfs;
}

/// Trait for upcasting a reference to a base trait object.
pub trait Upcast<T: ?Sized> {
    fn upcast(&self) -> &T;
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use salsa::DebugWithDb;

    use crate::file_system::{FileSystem, MemoryFileSystem};
    use crate::vfs::{VendoredPathBuf, Vfs};
    use crate::{Db, Jar};

    /// Database that can be used for testing.
    ///
    /// Uses an in memory filesystem and it stubs out the vendored files by default.
    #[salsa::db(Jar)]
    pub(crate) struct TestDb {
        storage: salsa::Storage<Self>,
        vfs: Vfs,
        file_system: MemoryFileSystem,
        events: std::sync::Arc<std::sync::Mutex<Vec<salsa::Event>>>,
    }

    impl TestDb {
        pub(crate) fn new() -> Self {
            let mut vfs = Vfs::default();
            vfs.stub_vendored::<VendoredPathBuf, String>([]);

            Self {
                storage: salsa::Storage::default(),
                file_system: MemoryFileSystem::default(),
                events: std::sync::Arc::default(),
                vfs,
            }
        }

        #[allow(unused)]
        pub(crate) fn file_system(&self) -> &MemoryFileSystem {
            &self.file_system
        }

        /// Empties the internal store of salsa events that have been emitted,
        /// and returns them as a `Vec` (equivalent to [`std::mem::take`]).
        ///
        /// ## Panics
        /// If there are pending database snapshots.
        #[allow(unused)]
        pub(crate) fn take_salsa_events(&mut self) -> Vec<salsa::Event> {
            let inner = Arc::get_mut(&mut self.events)
                .expect("expected no pending salsa database snapshots.");

            std::mem::take(inner.get_mut().unwrap())
        }

        /// Clears the emitted salsa events.
        ///
        /// ## Panics
        /// If there are pending database snapshots.
        #[allow(unused)]
        pub(crate) fn clear_salsa_events(&mut self) {
            self.take_salsa_events();
        }

        pub(crate) fn file_system_mut(&mut self) -> &mut MemoryFileSystem {
            &mut self.file_system
        }

        pub(crate) fn vfs_mut(&mut self) -> &mut Vfs {
            &mut self.vfs
        }
    }

    impl Db for TestDb {
        fn file_system(&self) -> &dyn FileSystem {
            &self.file_system
        }

        fn vfs(&self) -> &Vfs {
            &self.vfs
        }
    }

    impl salsa::Database for TestDb {
        fn salsa_event(&self, event: salsa::Event) {
            tracing::trace!("event: {:?}", event.debug(self));
            let mut events = self.events.lock().unwrap();
            events.push(event);
        }
    }

    impl salsa::ParallelDatabase for TestDb {
        fn snapshot(&self) -> salsa::Snapshot<Self> {
            salsa::Snapshot::new(Self {
                storage: self.storage.snapshot(),
                file_system: self.file_system.snapshot(),
                vfs: self.vfs.snapshot(),
                events: self.events.clone(),
            })
        }
    }
}
