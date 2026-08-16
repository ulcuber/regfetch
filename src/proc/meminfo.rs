use std::fs;

#[derive(Debug, Clone, Default)]
pub struct MemInfo {
    pub mem_total: u64,
    pub mem_free: u64,
    pub mem_available: u64,
    pub buffers: u64,
    pub cached: u64,
    pub swap_cached: u64,
    pub active: u64,
    pub inactive: u64,
    pub active_anon: u64,
    pub inactive_anon: u64,
    pub active_file: u64,
    pub inactive_file: u64,
    pub unevictable: u64,
    pub mlocked: u64,
    pub swap_total: u64,
    pub swap_free: u64,
    pub dirty: u64,
    pub writeback: u64,
    pub anon_pages: u64,
    pub mapped: u64,
    pub shmem: u64,
    pub k_reclaimable: u64,
    pub slab: u64,
    pub s_reclaimable: u64,
    pub s_unreclaim: u64,
    pub kernel_stack: u64,
    pub page_tables: u64,
    pub sec_page_tables: u64,
    pub nfs_unstable: u64,
    pub bounce: u64,
    pub writeback_tmp: u64,
    pub commit_limit: u64,
    pub committed_as: u64,
    pub vmalloc_total: u64,
    pub vmalloc_used: u64,
    pub vmalloc_chunk: u64,
    pub percpu: u64,
    pub hardware_corrupted: u64,
    pub anon_huge_pages: u64,
    pub shmem_huge_pages: u64,
    pub shmem_pmd_mapped: u64,
    pub file_huge_pages: u64,
    pub file_pmd_mapped: u64,
    pub cma_total: u64,
    pub cma_free: u64,
    pub hugepages_total: u64,
    pub hugepages_free: u64,
    pub hugepages_rsvd: u64,
    pub hugepages_surp: u64,
    pub hugepagesize: u64,
    pub direct_map_4k: u64,
    pub direct_map_2m: u64,
    pub direct_map_1g: u64,
    pub hugetlb: u64,
    pub per_cpu: u64,
}

impl MemInfo {
    pub fn read() -> Result<Self, String> {
        let content = fs::read_to_string("/proc/meminfo")
            .map_err(|e| format!("Failed to read /proc/meminfo: {}", e))?;

        Self::parse(&content)
    }

    pub fn parse(content: &str) -> Result<Self, String> {
        let mut meminfo = MemInfo {
            ..Default::default()
        };

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }

            let key = parts[0].trim_end_matches(':');
            // KB to B
            let value = parts[1].parse::<u64>().unwrap_or(0) * 1024;

            match key {
                "MemTotal" => meminfo.mem_total = value,
                "MemFree" => meminfo.mem_free = value,
                "MemAvailable" => meminfo.mem_available = value,
                "Buffers" => meminfo.buffers = value,
                "Cached" => meminfo.cached = value,
                "SwapCached" => meminfo.swap_cached = value,
                "Active" => meminfo.active = value,
                "Inactive" => meminfo.inactive = value,
                "Active(anon)" => meminfo.active_anon = value,
                "Inactive(anon)" => meminfo.inactive_anon = value,
                "Active(file)" => meminfo.active_file = value,
                "Inactive(file)" => meminfo.inactive_file = value,
                "Unevictable" => meminfo.unevictable = value,
                "Mlocked" => meminfo.mlocked = value,
                "SwapTotal" => meminfo.swap_total = value,
                "SwapFree" => meminfo.swap_free = value,
                "Dirty" => meminfo.dirty = value,
                "Writeback" => meminfo.writeback = value,
                "AnonPages" => meminfo.anon_pages = value,
                "Mapped" => meminfo.mapped = value,
                "Shmem" => meminfo.shmem = value,
                "KReclaimable" => meminfo.k_reclaimable = value,
                "Slab" => meminfo.slab = value,
                "SReclaimable" => meminfo.s_reclaimable = value,
                "SUnreclaim" => meminfo.s_unreclaim = value,
                "KernelStack" => meminfo.kernel_stack = value,
                "PageTables" => meminfo.page_tables = value,
                "SecPageTables" => meminfo.sec_page_tables = value,
                "NFS_Unstable" => meminfo.nfs_unstable = value,
                "Bounce" => meminfo.bounce = value,
                "WritebackTmp" => meminfo.writeback_tmp = value,
                "CommitLimit" => meminfo.commit_limit = value,
                "Committed_AS" => meminfo.committed_as = value,
                "VmallocTotal" => meminfo.vmalloc_total = value,
                "VmallocUsed" => meminfo.vmalloc_used = value,
                "VmallocChunk" => meminfo.vmalloc_chunk = value,
                "Percpu" => meminfo.percpu = value,
                "HardwareCorrupted" => meminfo.hardware_corrupted = value,
                "AnonHugePages" => meminfo.anon_huge_pages = value,
                "ShmemHugePages" => meminfo.shmem_huge_pages = value,
                "ShmemPmdMapped" => meminfo.shmem_pmd_mapped = value,
                "FileHugePages" => meminfo.file_huge_pages = value,
                "FilePmdMapped" => meminfo.file_pmd_mapped = value,
                "CmaTotal" => meminfo.cma_total = value,
                "CmaFree" => meminfo.cma_free = value,
                "HugePages_Total" => meminfo.hugepages_total = value,
                "HugePages_Free" => meminfo.hugepages_free = value,
                "HugePages_Rsvd" => meminfo.hugepages_rsvd = value,
                "HugePages_Surp" => meminfo.hugepages_surp = value,
                "Hugepagesize" => meminfo.hugepagesize = value,
                "DirectMap4k" => meminfo.direct_map_4k = value,
                "DirectMap2M" => meminfo.direct_map_2m = value,
                "DirectMap1G" => meminfo.direct_map_1g = value,
                "Hugetlb" => meminfo.hugetlb = value,
                "PerCPU" => meminfo.per_cpu = value,
                _ => {},
            }
        }

        Ok(meminfo)
    }
}
