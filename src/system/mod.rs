pub mod archivist;

pub use crate::context::{Partition, PartitionType, SystemContext};
pub mod discovery;
pub mod env_scanner;
pub mod global;
pub mod healer;

pub mod storage;
pub mod virt;

/// Sovereign SRE Blacklist: Directories to skip during wide system scans to avoid toil and noise.
pub const SRE_BLACKLIST: &[&str] = &[
    "/mnt/HDD/timeshift",
    "/proc",
    "/sys",
    "/dev",
    "/run",
    "/var/lib/docker",
    "/lost+found",
    "/usr/share/help", // Potential noise
    ".snapshots",
];
