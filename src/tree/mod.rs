mod region;

pub use region::{Region, RegionVariant};

#[derive(Debug)]
pub struct MemoryTree {
    roots: Vec<Region>,
    pub ram_size: u64,
    pub kernel_size: u64,
}

impl MemoryTree {
    pub fn new() -> Self {
        MemoryTree {
            roots: Vec::new(),
            ram_size: 0,
            kernel_size: 0,
        }
    }

    pub fn insert_region(&mut self, region: Region) {
        let mut part: Region = region;

        for root in self.roots.iter_mut() {
            if root.contains(part.start) {
                if root.contains(part.end) {
                    root.insert_region(part);

                    return;
                } else {
                    let left_part = Region::new(
                        part.start, root.end, part.variant, format!("{} (left part)", part.name)
                    );
                    let right_part = Region::new(
                        root.end + 1, part.end, part.variant, format!("{} (right part)", part.name)
                    );

                    root.insert_region(left_part);
                    part = right_part;

                    continue;
                }
            } else if root.contains(part.end) {
                let left_part = Region::new(
                    part.start, root.start - 1, part.variant, format!("{} (left part)", part.name)
                );
                let right_part = Region::new(
                    root.start, part.end, part.variant, format!("{} (right part)", part.name)
                );

                root.insert_region(left_part);
                root.insert_region(right_part);

                return;
            }
        }

        self.roots.push(part);
    }

    pub fn sort_by_start(&mut self) {
        self.roots.sort_by_key(|r| r.start);

        for root in self.roots.iter_mut() {
            root.sort_by_start();
        }
    }

    pub fn print_tree(&self) {
        for region in &self.roots {
            region.print_tree(0);
        }
    }
}
