use crate::system::virt::VirtualMachine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Partition {
    pub mount_point: String,
    pub filesystem: String,
    pub total_size: String,
    pub used: String,
    pub available: String,
    pub usage_percent: String,
    pub partition_type: PartitionType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PartitionType {
    Root,
    User,
    Media,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemContext {
    pub os_name: String,
    pub kernel_version: String,
    pub partitions: Vec<Partition>,
    pub vms: Vec<VirtualMachine>,
    pub env_vars: HashMap<String, String>,
    pub plugin_manager: Option<String>,
    pub ssh_auth_sock: Option<String>,
}

impl SystemContext {
    pub fn new() -> Self {
        SystemContext {
            os_name: "Unknown".to_string(),
            kernel_version: "Unknown".to_string(),
            partitions: Vec::new(),
            vms: Vec::new(),
            env_vars: HashMap::new(),
            plugin_manager: None,
            ssh_auth_sock: None,
        }
    }
}
