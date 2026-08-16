use std::fs;

use crate::tree::{MemoryTree, Region, RegionVariant};

pub fn read_iomem(tree: &mut MemoryTree) -> Result<(), String> {
    let iomem_content = fs::read_to_string("/proc/iomem")
        .map_err(|e| format!("Failed to read /proc/iomem: {}", e))?;

    parse_iomem(tree, &iomem_content)?;

    Ok(())
}

fn parse_iomem(tree: &mut MemoryTree, content: &str) -> Result<(), String> {
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let indent = line.len() - line.trim_start().len();
        let depth = indent / 2;

        if let Some(region) = parse_iomem_line(line.trim()) {
            if depth == 0 {
                tree.ram_size += region.size;
            }

            match region.variant {
                RegionVariant::KernelCode|RegionVariant::KernelRodata|RegionVariant::KernelData|RegionVariant::KernelBss => {
                    tree.kernel_size += region.size;
                },
                _ => (),
            };

            tree.insert_region(region);
        }
    }

    Ok(())
}

fn parse_iomem_line(line: &str) -> Option<Region> {
    let parts: Vec<&str> = line.splitn(2, ':').collect();
    if parts.len() != 2 {
        eprintln!("Cannot split /proc/iomem line");
        return None;
    }

    let range = parts[0].trim();
    let name = parts[1].trim().to_string();

    let addr_parts: Vec<&str> = range.split('-').collect();
    if addr_parts.len() != 2 {
        eprintln!("Cannot split /proc/iomem address range");
        return None;
    }

    let start = u64::from_str_radix(addr_parts[0].trim(), 16).ok()?;
    let end = u64::from_str_radix(addr_parts[1].trim(), 16).ok()?;

    if let Some(v) = detect_iomem_variant(&name) {
        return Some(Region::new(start, end, v, name));
    }

    None
}

fn detect_iomem_variant(name: &str) -> Option<RegionVariant> {
    if name == "Kernel code" {
        Some(RegionVariant::KernelCode)
    } else if name == "Kernel rodata" {
        Some(RegionVariant::KernelRodata)
    } else if name == "Kernel data" {
        Some(RegionVariant::KernelData)
    } else if name == "Kernel bss" {
        Some(RegionVariant::KernelBss)
    } else if name == "System RAM" {
        Some(RegionVariant::SystemRam)
    } else {
        // PCI, Reserved, etc
        None
    }
}
