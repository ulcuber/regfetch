use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::util::human_readable_size as h;

#[derive(Debug, Clone)]
pub struct Proc {
    pub pid: u32,
    pub ppid: u32,
    pub private_size: u64, // Private_Clean + Private_Dirty of anon, [heap], [stack], etc
    pub total_private: u64, // With children
    pub pss: u64,
    pub total_pss: u64, // With children
    pub name: String,
    pub children: Vec<Proc>,
}

impl Proc {
    pub fn new(pid: u32, name: String) -> Self {
        Proc {
            pid,
            ppid: 0,
            private_size: 0,
            total_private: 0,
            pss: 0,
            total_pss: 0,
            name,
            children: Vec::new(),
        }
    }

    pub fn update_totals(&mut self, own_pid: u32) -> Option<Proc> {
        let mut own_proc: Option<Proc> = None;

        self.total_private = self.private_size;
        self.total_pss = self.pss;
        for child in &mut self.children {
            let own = child.update_totals(own_pid);
            if own.is_some() {
                own_proc = own;
            }
            self.total_private += child.total_private;
            self.total_pss += child.total_pss;

            if child.pid == std::process::id() {
                own_proc = Some(child.clone());
            }
        }

        own_proc
    }

    pub fn sort_by_size(&mut self) {
        self.children.sort_by_key(|r| std::cmp::Reverse(r.total_private));

        for child in self.children.iter_mut() {
            child.sort_by_size();
        }
    }

    pub fn print_tree(&self, depth: usize) {
        let indent = " ".repeat(depth * 2);
        println!(
            "{} {} {} ({}/{}, {}/{})",
            indent,
            self.pid,
            self.name,
            h(self.private_size),
            h(self.pss),
            h(self.total_private),
            h(self.total_pss),
        );
        for child in &self.children {
            child.print_tree(depth + 1);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Exe {
    pub total_size: u64, // Shared_Clean + Shared_Dirty of /**/* for each proc
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Lib {
    pub total_size: u64, // Shared_Clean + Shared_Dirty of /**/*.so for each proc
    pub name: String,
}

#[derive(Debug)]
pub struct ProcTree {
    pub total_private: u64,
    pub total_pss: u64,
    pub total_exes: u64,
    pub total_libs: u64,
    pub own_proc: Option<Proc>,
    pub roots: Vec<Proc>,
    pub sorted_exes: Vec<Exe>,
    pub sorted_libs: Vec<Lib>,
    pub exes: HashMap<String, Exe>,
    pub libs: HashMap<String, Lib>,
    pub addresses: HashSet<u64>, // start addresses to prevent double count
}

impl ProcTree {
    pub fn new() -> Self {
        ProcTree {
            total_private: 0,
            total_pss: 0,
            total_exes: 0,
            total_libs: 0,
            own_proc: None,
            roots: Vec::new(),
            sorted_exes: Vec::new(),
            sorted_libs: Vec::new(),
            exes: HashMap::new(),
            libs: HashMap::new(),
            addresses: HashSet::new(),
        }
    }

    pub fn read() -> Result<Self, String> {
        let mut tree = ProcTree::new();

        tree.parse()?;

        Ok(tree)
    }

    pub fn parse(&mut self) -> Result<(), String> {
        let proc_dir = Path::new("/proc");

        let mut procs_by_pid: HashMap<u32, Proc> = HashMap::new();
        for entry in fs::read_dir(proc_dir).map_err(|e| format!("Failed to read /proc: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();
            if let Some(pid_str) = path.file_name().and_then(|n| n.to_str()) {
                if let Ok(pid) = pid_str.parse::<u32>() {
                    if pid > 0 && path.join("smaps").exists() {
                        match self.parse_process(pid) {
                            Ok(proc) => {
                                procs_by_pid.insert(pid, proc);
                            },
                            Err(e) => {
                                eprintln!("{}", e);
                            },
                        };
                    }
                }
            }
        }

        let mut children_by_ppid: HashMap<u32, Vec<Proc>> = HashMap::new();
        for (_, proc) in procs_by_pid.drain() {
            children_by_ppid.entry(proc.ppid).or_default().push(proc);
        }

        if let Some(roots) = children_by_ppid.remove(&0) {
            self.roots = roots;
        }

        let mut queue: Vec<&mut Proc> = self.roots.iter_mut().collect();

        while let Some(current) = queue.pop() {
            if let Some(children) = children_by_ppid.remove(&current.pid) {
                current.children = children;
                queue.extend(current.children.iter_mut());
            }
        }

        let own_pid = std::process::id();
        for root in &mut self.roots {
            let own_proc = root.update_totals(own_pid);
            if own_proc.is_some() {
                self.own_proc = own_proc;
            }
            self.total_private += root.total_private;
            self.total_pss += root.total_pss;
        }

        Ok(())
    }

    pub fn parse_process(&mut self, pid: u32) -> Result<Proc, String> {
        let name = Self::get_process_name(pid)?;

        let mut proc = Proc::new(pid, name);

        let smaps_path = format!("/proc/{}/smaps", pid);
        if let Ok(content) = fs::read_to_string(&smaps_path) {
            let mut current_region = String::new();
            let mut private_clean = 0u64;
            let mut private_dirty = 0u64;
            let mut pss = 0u64;
            let mut shared_clean = 0u64;
            let mut shared_dirty = 0u64;
            let mut is_anon = false;
            let mut is_exe = false;
            let mut is_lib = false;
            let mut start_addr = 0u64;

            for line in content.lines() {
                let trimmed = line.trim();

                if trimmed.contains(" r") && trimmed.contains("p ") && trimmed.contains("-") {
                    if !current_region.is_empty() && !self.addresses.contains(&start_addr) {
                        if is_anon {
                            proc.private_size += private_clean + private_dirty;
                            proc.pss += pss;
                        } else if is_exe {
                            let shared = shared_clean + shared_dirty;
                            self.total_exes += shared;
                            if let Some(exe) = self.exes.get_mut(&current_region) {
                                exe.total_size += shared;
                            } else {
                                self.exes.insert(current_region.clone(), Exe {
                                    total_size: shared,
                                    name: current_region,
                                });
                            }
                        } else if is_lib {
                            let shared = shared_clean + shared_dirty;
                            self.total_libs += shared;
                            if let Some(lib) = self.libs.get_mut(&current_region) {
                                lib.total_size += shared;
                            } else {
                                self.libs.insert(current_region.clone(), Lib {
                                    total_size: shared,
                                    name: current_region,
                                });
                            }
                        }
                    }

                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() < 5 {
                        return Err(format!("Expected 5 or more parts"));
                    }

                    let addr_parts: Vec<&str> = parts[0].split('-').collect();
                    if addr_parts.len() == 2 {
                        if let Ok(addr) = u64::from_str_radix(addr_parts[0], 16) {
                            start_addr = addr;
                        }
                    }

                    // let flags = parts[1];
                    // 2 offset in file
                    // 3 device hosting the file
                    // let inode = u64::from_str_radix(parts[4], 10).ok();
                    let path = parts[5..].join(" ");

                    is_anon = path.is_empty() || path.starts_with('[');
                    is_exe = path.starts_with('/') && !path.contains(".so");
                    is_lib = path.contains(".so");

                    current_region = path;
                    private_clean = 0;
                    private_dirty = 0;
                    pss = 0;
                    shared_clean = 0;
                    shared_dirty = 0;
                } else if trimmed.starts_with("Private_Clean:") {
                    if let Some(val) = Self::parse_smaps_value(trimmed) {
                        private_clean = val;
                    }
                } else if trimmed.starts_with("Private_Dirty:") {
                    if let Some(val) = Self::parse_smaps_value(trimmed) {
                        private_dirty = val;
                    }
                } else if trimmed.starts_with("Pss:") {
                    if let Some(val) = Self::parse_smaps_value(trimmed) {
                        pss = val;
                    }
                } else if trimmed.starts_with("Shared_Clean:") {
                    if let Some(val) = Self::parse_smaps_value(trimmed) {
                        shared_clean = val;
                    }
                } else if trimmed.starts_with("Shared_Dirty:") {
                    if let Some(val) = Self::parse_smaps_value(trimmed) {
                        shared_dirty = val;
                    }
                }
            }

            if !current_region.is_empty() && !self.addresses.contains(&start_addr) {
                if is_anon {
                    proc.private_size += private_clean + private_dirty;
                    proc.pss += pss;
                } else if is_exe {
                    let shared = shared_clean + shared_dirty;
                    self.total_exes += shared;
                    if let Some(exe) = self.exes.get_mut(&current_region) {
                        exe.total_size += shared;
                    } else {
                        self.exes.insert(current_region.clone(), Exe {
                            total_size: shared,
                            name: current_region,
                        });
                    }
                } else if is_lib {
                    let shared = shared_clean + shared_dirty;
                    self.total_libs += shared;
                    if let Some(lib) = self.libs.get_mut(&current_region) {
                        lib.total_size += shared;
                    } else {
                        self.libs.insert(current_region.clone(), Lib {
                            total_size: shared,
                            name: current_region,
                        });
                    }
                }
            }
        }

        let status_path = format!("/proc/{}/status", pid);
        if let Ok(content) = fs::read_to_string(&status_path) {
            for line in content.lines() {
                if line.starts_with("PPid:") {
                    if let Some(ppid) = Self::parse_meminfo_value(line) {
                        proc.ppid = ppid as u32;
                    }
                }
            }
        }

        Ok(proc)
    }

    fn get_process_name(pid: u32) -> Result<String, String> {
        let comm_path = format!("/proc/{}/comm", pid);
        if let Ok(content) = fs::read_to_string(&comm_path) {
            return Ok(content.trim().to_string());
        }

        let cmdline_path = format!("/proc/{}/cmdline", pid);
        if let Ok(content) = fs::read_to_string(&cmdline_path) {
            let name = content.split('\0').next().unwrap_or("").to_string();
            if !name.is_empty() {
                return Ok(name);
            }
        }

        Ok(format!("[{}]", pid))
    }

    fn parse_smaps_value(line: &str) -> Option<u64> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(val) = parts[1].parse::<u64>() {
                // Values in smaps are in kB
                return Some(val * 1024);
            }
        }
        None
    }

    fn parse_meminfo_value(line: &str) -> Option<u64> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(val) = parts[1].parse::<u64>() {
                // Values in /proc/pid/status are in kB
                return Some(val);
            }
        }
        None
    }

    pub fn sort_by_size(&mut self) {
        self.roots.sort_by_key(|r| std::cmp::Reverse(r.total_private));

        for root in self.roots.iter_mut() {
            root.sort_by_size();
        }

        self.sorted_exes = self.exes.clone().into_values().collect::<Vec<Exe>>();
        self.sorted_exes.sort_by_key(|i| std::cmp::Reverse(i.total_size));

        self.sorted_libs = self.libs.clone().into_values().collect::<Vec<Lib>>();
        self.sorted_libs.sort_by_key(|i| std::cmp::Reverse(i.total_size));
    }

    pub fn print_tree(&self) {
        for proc in &self.roots {
            proc.print_tree(0);
        }
        println!(
            "Procs total: {}/{}",
            h(self.total_private),
            h(self.total_pss),
        );
    }

    pub fn print_exes(&self) {
        for exe in &self.sorted_exes {
            println!("{} {}", exe.name, h(exe.total_size));
        }

        println!("Total exes: {}", h(self.total_exes));
    }

    pub fn print_libs(&self) {
        for lib in &self.sorted_libs {
            println!("{} {}", lib.name, h(lib.total_size));
        }

        println!("Total libs: {}", h(self.total_libs));
    }
}
