# About

- Shows physical System RAM (`/proc/iomem`) regions alignment within memory zones (`/proc/zoneinfo`: `DMA`, `DMA32`, `Normal`)
- Shows Kernel modules memory usage (`/proc/modules`)
- Shows processes hierarchy with memory usage (`/proc/<pid>/{status|smaps}`), executables and libs memory
- Shows memory usage hierarchy (`/proc/meminfo` + calculated)
- Data sorted by size to detect the most heavy consumers

# Install

```bash
cargo build --release
sudo cp --update=all target/release/regfetch /usr/bin
```

# Run

```bash
sudo regfetch -h
sudo regfetch -a
sudo regfetch -a | less
```
