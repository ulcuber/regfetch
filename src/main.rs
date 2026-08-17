use regfetch::{
    tree::MemoryTree,
    util::human_readable_size as h,
    proc::{
        read_zoneinfo,
        read_iomem,
        MemInfo,
        KernelModules,
        ProcTree,
    },
};

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);

    let mut show_regs = false;
    let mut show_mods = false;
    let mut show_procs = false;
    let mut show_exes = false;
    let mut show_libs = false;
    let mut show_total = true;
    let mut show_compact = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("Usage: program [options]");
                println!("  -h, --help       Show help");
                println!("  -r, --regs       Show /proc/zoneinfo + /proc/iomem");
                println!("  -m, --mods       Show /proc/modules");
                println!("  -p, --procs      Show /proc/<pid>/smaps + /proc/<pid>/status");
                println!("  -e, --exes       Show /proc/<pid>/smaps + /proc/<pid>/status executables");
                println!("  -l, --libs       Show /proc/<pid>/smaps + /proc/<pid>/status libraries");
                println!("  -a, --all        Show all of the above");
                println!("  -n, --no-total   Do not show totals (/proc/meminfo + calculated)");
                println!("  -c, --compact    Show minimum info");
                return Ok(());
            },
            "-r" | "--regs" => {
                show_regs = true;
            },
            "-m" | "--mods" => {
                show_mods = true;
            },
            "-p" | "--procs" => {
                show_procs = true;
            },
            "-e" | "--exes" => {
                show_exes = true;
            },
            "-l" | "--libs" => {
                show_libs = true;
            },
            "-a" | "--all" => {
                show_regs = true;
                show_procs = true;
                show_mods = true;
                show_exes = true;
                show_libs = true;
                show_total = true;
            },
            "-n" | "--no-total" => {
                show_total = false;
            },
            "-c" | "--compact" => {
                show_total = false;
                show_compact = true;
            },
            _ => {
                println!("Unknown argument: {}", arg);
            },
        }
    }

    let mut tree = MemoryTree::new();

    read_zoneinfo(&mut tree)?;
    read_iomem(&mut tree)?;

    let mem = MemInfo::read()?;
    let mut kmods = KernelModules::read()?;
    let mut procs = ProcTree::read()?;

    tree.sort_by_start();
    kmods.sort_by_size();
    procs.sort_by_size();

    if show_regs {
        tree.print_tree();
    }

    if show_mods {
        println!();
        kmods.print_tree();
    }

    if show_procs {
        println!();
        procs.print_tree();
    }

    if show_exes {
        println!();
        procs.print_exes();
    }

    if show_libs {
        println!();
        procs.print_libs();
    }

    let own_private = if let Some(proc) = procs.own_proc {
        proc.total_private
    } else {
        0
    };

    if show_compact {
        let used = procs.total_private + procs.total_exes + procs.total_libs;
        let total = used + kmods.used;
        println!("Kernel out of MemTotal: {}", h(tree.kernel_size));
        println!("Used: {}", h(total));
        println!("  User: {}", h(used));
        println!("    Private: {} (see -p)", h(procs.total_private));
        println!("    Exes: {} (reduce use flags)", h(procs.total_exes));
        println!("    Libs: {} (use musl, see -l)", h(procs.total_libs));
        println!("  Kernel");
        println!("    Modules: {} (use =y instead of =m to move out of MemTotal, see -m)", h(kmods.used));
        println!("Own: {}", h(own_private));
    }

    if !show_total {
        return Ok(());
    }

    let system_ram_reserved = tree.ram_size - tree.kernel_size - mem.mem_total;

    let total_private = procs.total_private;
    let not_free = mem.mem_available - mem.mem_free;
    let used = mem.mem_total - mem.mem_available;

    let kernel = mem.s_unreclaim + mem.kernel_stack + mem.page_tables + mem.sec_page_tables;
    let mut other_used = used - total_private - procs.total_exes - procs.total_libs - kernel - mem.vmalloc_used;

    let mut unused_cache = mem.cached - other_used;
    let mut other_not_free = not_free - mem.buffers - mem.k_reclaimable;
    if unused_cache > other_not_free {
        let delta = unused_cache - other_not_free;
        unused_cache -= delta;
        other_used += delta;
    }
    other_not_free -= unused_cache;

    println!();
    println!("RAM size: {}", h(tree.ram_size));
    println!("  Kernel: {}", h(tree.kernel_size));
    println!("  Reserved: {}", h(system_ram_reserved));
    println!("  Total: {}", h(mem.mem_total));
    println!("    Available: {}", h(mem.mem_available));
    println!("      Free: {}", h(mem.mem_free));
    println!("      Not free: {}", h(not_free));
    println!("        Buffers: {}", h(mem.buffers));
    println!("        KReclaimable: {}", h(mem.k_reclaimable));
    println!("          SReclaimable: {} ({})", h(mem.s_reclaimable), h(mem.slab));
    println!("        Unused cache: {} ({})", h(unused_cache), h(mem.cached));
    println!("        Other: {}", h(other_not_free));
    println!("    Used: {}", h(used));
    println!("      Private: {}", h(total_private));
    println!("        Own private: {}", h(own_private));
    println!("        Other: {}", h(total_private - own_private));
    println!("      Exes: {}", h(procs.total_exes));
    println!("      Libs: {}", h(procs.total_libs));
    println!("      Kernel: {}", h(kernel));
    println!("        SUnreclaim: {} ({})", h(mem.s_unreclaim), h(mem.slab));
    println!("        KernelStack: {}", h(mem.kernel_stack));
    println!("        PageTables: {}", h(mem.page_tables));
    println!("          [vdso]: {}", h(procs.total_vdso));
    println!("          [vvar]: {}", h(procs.total_vvar));
    println!("        SecPageTables: {}", h(mem.sec_page_tables));
    println!("      VmallocUsed: {}", h(mem.vmalloc_used));
    println!("        Modules: {}", h(kmods.used));
    println!("    Active cache: {} ({})", h(other_used), h(mem.cached));
    println!("      Shmem: {}", h(mem.shmem));

    Ok(())
}
