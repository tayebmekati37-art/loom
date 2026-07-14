#[derive(Debug, Clone)]
pub enum StorageClass {
    WorkingStorage,
    LocalStorage,
    Linkage,
    FileSection,
}
