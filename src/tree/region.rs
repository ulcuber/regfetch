use crate::util::human_readable_size;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionVariant {
    // /proc/zoneinfo
    ZoneDMA,
    ZoneDMA32,
    ZoneNormal,
    // /proc/iomem
    SystemRam,
    KernelCode,
    KernelRodata,
    KernelData,
    KernelBss,
    // /proc/modules
    KernelModule,
}

#[derive(Debug)]
pub struct Region {
    pub start: u64,
    pub end: u64,
    pub size: u64,
    pub used: u64,  // Total size of all descendants
    pub variant: RegionVariant,
    pub name: String,
    pub children: Vec<Region>,
}

impl Region {
    pub fn new(start: u64, end: u64, variant: RegionVariant, name: String) -> Self {
        Region {
            start,
            end,
            size: if start <= end { end - start + 1 } else { 0 },
            used: 0,
            variant,
            name,
            children: Vec::new(),
        }
    }

    pub fn contains(&self, addr: u64) -> bool {
        addr >= self.start && addr <= self.end
    }

    pub fn insert_region(&mut self, region: Region) {
        for child in self.children.iter_mut() {
            if child.contains(region.start) && child.contains(region.end) {
                child.insert_region(region);

                return;
            }
        }

        self.children.push(region);
    }

    pub fn sort_by_start(&mut self) {
        self.children.sort_by_key(|r| r.start);

        for child in self.children.iter_mut() {
            child.sort_by_start();
        }
    }

    pub fn print_tree(&self, depth: usize) {
        let indent = "  ".repeat(depth);
        let post_indent = "  ".repeat(5 - depth);
        let used = if self.used > 0 {
            format!("{}/", human_readable_size(self.used))
        } else {
            "".to_string()
        };
        let size = human_readable_size(self.size);
        println!("{}{:016x}-{:016x} {} {:>9}{:<9} {} {}",
            indent, self.start, self.end, post_indent, used, size, indent, self.name);
        for child in &self.children {
            child.print_tree(depth + 1);
        }
    }
}
