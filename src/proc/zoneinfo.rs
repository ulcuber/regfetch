use std::fs;

use crate::tree::{MemoryTree, Region, RegionVariant};

pub fn read_zoneinfo(tree: &mut MemoryTree) -> Result<(), String> {
    let iomem_content = fs::read_to_string("/proc/zoneinfo")
        .map_err(|e| format!("Failed to read /proc/zoneinfo: {}", e))?;

    parse_zoneinfo(tree, &iomem_content)?;

    Ok(())
}

fn parse_zoneinfo(tree: &mut MemoryTree, content: &str) -> Result<(), String> {
    const PAGE_SIZE: u64 = 4096;

    let mut current_zone: Option<ZoneInfo> = None;
    let mut current_zone_name = String::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("Node") && trimmed.contains("zone") {
            if let Some(zone) = current_zone.take() {
                add_zone_to_tree(tree, zone);
            }

            // Parse zone name
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if let Some(zone_name_pos) = parts.iter().position(|&p| p == "zone") {
                if zone_name_pos + 1 < parts.len() {
                    current_zone_name = parts[zone_name_pos + 1].to_string();
                }
            }

            current_zone = Some(ZoneInfo {
                name: current_zone_name.clone(),
                start_pfn: 0,
                spanned_pages: 0,
                present_pages: 0,
                managed_pages: 0,
                free_pages: 0,
                min_pages: 0,
                low_pages: 0,
                high_pages: 0,
                protection: Vec::new(),
                start_address: 0,
            });

            continue;
        }

        // Parse zone stats only if we're inside a zone
        if let Some(zone) = &mut current_zone {
            // Parse numeric values
            if trimmed.starts_with("pages free") {
                if let Some(val) = parse_value_after_colon(trimmed) {
                    zone.free_pages = val;
                }
            } else if trimmed.starts_with("spanned") {
                if let Some(val) = parse_value_after_colon(trimmed) {
                    zone.spanned_pages = val;
                }
            } else if trimmed.starts_with("present") {
                if let Some(val) = parse_value_after_colon(trimmed) {
                    zone.present_pages = val;
                }
            } else if trimmed.starts_with("managed") {
                if let Some(val) = parse_value_after_colon(trimmed) {
                    zone.managed_pages = val;
                }
            } else if trimmed.starts_with("min") {
                if let Some(val) = parse_value_after_colon(trimmed) {
                    zone.min_pages = val;
                }
            } else if trimmed.starts_with("low") {
                if let Some(val) = parse_value_after_colon(trimmed) {
                    zone.low_pages = val;
                }
            } else if trimmed.starts_with("high") {
                if let Some(val) = parse_value_after_colon(trimmed) {
                    zone.high_pages = val;
                }
            } else if trimmed.starts_with("protection") {
                if let Some(protection_str) = trimmed.split(':').nth(1) {
                    let protection_vals: Vec<u64> = protection_str
                        .trim()
                        .trim_start_matches('(')
                        .trim_end_matches(')')
                        .split(',')
                        .filter_map(|s| s.trim().parse::<u64>().ok())
                        .collect();
                    zone.protection = protection_vals;
                }
            } else if trimmed.starts_with("start_pfn:") {
                if let Some(val) = parse_value_after_colon(trimmed) {
                    zone.start_pfn = val;
                    // Calculate start address
                    zone.start_address = val * PAGE_SIZE;
                }
            } else if trimmed.starts_with("nr_free_pages") {
                if let Some(val) = parse_value_after_colon(trimmed) {
                    zone.free_pages = val;
                }
            }
        }
    }

    if let Some(zone) = current_zone.take() {
        add_zone_to_tree(tree, zone);
    }

    Ok(())
}

fn parse_value_after_colon(line: &str) -> Option<u64> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        if let Ok(val) = parts[1].parse::<u64>() {
            return Some(val);
        }
    }
    None
}

#[derive(Debug, Clone)]
struct ZoneInfo {
    name: String,
    start_pfn: u64,
    spanned_pages: u64,
    present_pages: u64,
    managed_pages: u64,
    free_pages: u64,
    min_pages: u64,
    low_pages: u64,
    high_pages: u64,
    protection: Vec<u64>,
    start_address: u64,
}

fn add_zone_to_tree(tree: &mut MemoryTree, zone: ZoneInfo) {
    const PAGE_SIZE: u64 = 4096;

    let start = zone.start_pfn * PAGE_SIZE;
    let end = start + (zone.spanned_pages * PAGE_SIZE) - 1;
    let total_size = zone.spanned_pages * PAGE_SIZE;
    let free_size = zone.free_pages * PAGE_SIZE;
    let used_size = total_size - free_size;


    if let Some(v) = detect_zoneinfo_variant(&zone.name) {
        let mut region = Region::new(start, end, v, zone.name);
        region.used = used_size;

        if region.size != total_size {
            eprintln!("Zone spanned_pages size != region size");
        }

        tree.insert_region(region);
    }
}

fn detect_zoneinfo_variant(name: &str) -> Option<RegionVariant> {
    match name {
        "DMA" => Some(RegionVariant::ZoneDMA),
        "DMA32" => Some(RegionVariant::ZoneDMA32),
        "Normal" => Some(RegionVariant::ZoneNormal),
        // "Movable" => None,
        // "Device" => None,
        _ => None,
    }
}
