use std::fs;

use crate::tree::{Region, RegionVariant};

#[derive(Debug, Default)]
pub struct KernelModules {
    // virtual addresses
    roots: Vec<Region>,
    pub used: u64,
}

impl KernelModules {
    pub fn read() -> Result<Self, String> {
        // aka /sys/module/name/ coresize refcnt initstate sections/.text
        // shows vmalloc size as VmallocUsed in /proc/meminfo
        let modules_content = fs::read_to_string("/proc/modules")
            .map_err(|e| format!("Failed to read /proc/modules: {}", e))?;

        let mut kmods = Self {
            ..Default::default()
        };
        kmods.parse(&modules_content)?;

        Ok(kmods)
    }

    pub fn parse(&mut self, content: &str) -> Result<(), String> {
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 4 {
                eprintln!("Cannot split /proc/modules line");
                continue;
            }

            let name = parts[0].to_string();
            let size: u64 = parts[1].parse().unwrap_or(0);
            // 2 refs
            // 3 deps
            // 4 Live
            let offset = parts[5];

            if let Ok(start) = u64::from_str_radix(&offset[2..], 16) {
                let end = start + size - 1;
                let region = Region::new(start, end, RegionVariant::KernelModule, name);

                self.used += size;

                self.roots.push(region);
            } else {
                eprintln!("Cannot parse address {} in /proc/modules line", offset);
            }
        }

        Ok(())
    }

    pub fn sort_by_size(&mut self) {
        self.roots.sort_by_key(|r| std::cmp::Reverse(r.size));
    }

    pub fn print_tree(&self) {
        for region in &self.roots {
            region.print_tree(0);
        }
    }
}
