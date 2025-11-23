use udev::Enumerator;
use std::fs;
use nix::sys::statvfs::statvfs;

#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub model: String,
    pub total_space: u64,
    pub partitions: Vec<PartitionInfo>,
    pub is_cdrom: bool,
    pub icon_name: String,
}

#[derive(Debug, Clone)]
pub struct PartitionInfo {
    pub device: String,
    pub mount_point: String,
    pub total_space: u64,
    pub used_space: u64,
    pub file_system: String,
}

#[derive(Debug, Clone)]
pub enum Drive {
    Local(DiskInfo),
    Network(NetworkDrive),
}

#[derive(Debug, Clone)]
pub struct NetworkDrive {
    pub mount_point: String,
    pub used_space: u64,
    pub total_space: u64,
}

pub async fn scan_disks() -> Vec<Drive> {
    let verbose = std::env::args().any(|arg| arg == "--verbose");
    let mut drives = Vec::new();

    // Local disks via udev
    let mut enumerator = Enumerator::new().expect("Failed to initialize udev");
    enumerator.match_subsystem("block").expect("Failed to set subsystem filter");

    let devices: Vec<_> = enumerator.scan_devices().expect("Failed to scan udev devices").collect();
    let mut root_map: Vec<(String, Vec<PartitionInfo>, bool, String)> = Vec::new();

    let mounts = fs::read_to_string("/proc/mounts").expect("Failed to read /proc/mounts");
    let mounted: Vec<(String, String, String)> = mounts
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 && parts[0].starts_with("/dev/") {
                let mp = parts[1].replace("\\040", " ");
                Some((parts[0].to_string(), mp, parts[2].to_string()))
            } else {
                None
            }
        })
        .collect();

    for device in &devices {
        if let Some(devnode) = device.devnode() {
            let dev_path = devnode.to_string_lossy().to_string();
            if !dev_path.starts_with("/dev/") {
                continue;
            }

            let blacklist = ["zram", "snd", "drm", "cpu", "hid"];
            if blacklist.iter().any(|&b| dev_path.contains(b)) {
                continue;
            }

            let mount_info = mounted.iter().find(|(dev, _, _)| dev == &dev_path);
            let (mount_point, file_system) = match mount_info {
                Some((_, mp, fs)) => (mp.clone(), fs.clone()),
                None => {
                    if verbose {
                        println!("Skipping {} - not mounted", dev_path);
                    }
                    continue;
                }
            };

            if file_system.is_empty() || file_system == "unknown" {
                if verbose {
                    println!("Skipping {} - no filesystem", dev_path);
                }
                continue;
            }

            let is_cdrom = dev_path.contains("sr") || dev_path.contains("cdrom");
            let is_root = if dev_path.contains("nvme") {
                !dev_path.contains('p')
            } else if is_cdrom {
                true
            } else {
                !dev_path.chars().last().unwrap_or(' ').is_digit(10)
            };

            let root_dev = if is_root {
                dev_path.clone()
            } else if dev_path.contains("nvme") {
                dev_path.split('p').next().unwrap().to_string()
            } else {
                format!("/dev/{}", dev_path.split('/').last().unwrap().chars().take_while(|c| !c.is_digit(10)).collect::<String>())
            };

            let (total_space, used_space) = if is_cdrom {
                match statvfs(mount_point.as_str()) {
                    Ok(stat) => {
                        let block_size = stat.block_size() as u64;
                        let total = stat.blocks() * block_size;
                        (total, total)
                    }
                    Err(_) => {
                        let size_str = device.property_value("SIZE").map(|s| s.to_string_lossy().to_string()).unwrap_or("0".to_string());
                        let size = size_str.parse::<u64>().unwrap_or(0) * 512;
                        if verbose {
                            println!("CDROM fallback for {}: SIZE={}", dev_path, size);
                        }
                        (size, size)
                    }
                }
            } else {
                match statvfs(mount_point.as_str()) {
                    Ok(stat) => {
                        let block_size = stat.block_size() as u64;
                        let total = stat.blocks() * block_size;
                        let free = stat.blocks_free() * block_size;
                        (total, total - free)
                    }
                    Err(e) => {
                        if verbose {
                            println!("Failed to statvfs {} ({}): {:?}", dev_path, mount_point, e);
                        }
                        (0, 0)
                    }
                }
            };

            let icon_name = if is_cdrom {
                "drive-optical"
            } else {
                let media = device.property_value("ID_DRIVE_MEDIA").map(|s| s.to_string_lossy().to_string());
                match media.as_deref() {
                    Some("solidstate") => "drive-harddisk-solidstate",
                    Some("disk") => "drive-harddisk",
                    Some("flash") | Some("usb") => "drive-removable-media",
                    Some("floppy") => "drive-floppy",
                    _ => if device.property_value("ID_BUS").map_or(false, |b| b == "usb") {
                        "drive-removable-media"
                    } else {
                        "drive-harddisk"
                    }
                }
            };

            let partition = PartitionInfo {
                device: dev_path.clone(),
                mount_point,
                total_space,
                used_space,
                file_system: file_system.clone(),
            };

            if verbose {
                println!("udev saw: {} (fs: {}) -> root: {} (icon: {})", dev_path, file_system, root_dev, icon_name);
            }

            if let Some((_, partitions, _, _)) = root_map.iter_mut().find(|(root, _, _, _)| root == &root_dev) {
                partitions.push(partition);
            } else {
                root_map.push((root_dev, vec![partition], is_cdrom, icon_name.to_string()));
            }
        }
    }
    if verbose {
        println!("udev found {} root devices with mounted partitions", root_map.len());
    }

    for (root_dev, partitions, is_cdrom, mut icon_name) in root_map {
        let total_space = partitions.iter().map(|p| p.total_space).sum();
        let mut model = "Unknown Model".to_string();

        for device in &devices {
            if let Some(devnode) = device.devnode() {
                let dev_path = devnode.to_string_lossy().to_string();
                if dev_path != root_dev {
                    continue;
                }

                model = if is_cdrom {
                    device
                        .property_value("ID_FS_LABEL")
                        .map(|s| s.to_string_lossy().to_string().replace("_", " "))
                        .unwrap_or_else(|| {
                            device
                                .property_value("ID_MODEL")
                                .map(|s| s.to_string_lossy().to_string().replace("_", " "))
                                .unwrap_or("Unknown Disc".to_string())
                        })
                } else {
                    device
                        .property_value("ID_MODEL")
                        .or_else(|| device.property_value("ID_MODEL_ID"))
                        .map(|s| s.to_string_lossy().to_string().replace("_", " "))
                        .unwrap_or("Unknown Model".to_string())
                };

                // Force icon_name for CDROMs
                icon_name = if is_cdrom {
                    "drive-optical".to_string()
                } else {
                    icon_name
                };

                if verbose {
                    println!("udev matched: {} -> {} (icon: {})", dev_path, model, icon_name);
                }
                break;
            }
        }

        if verbose {
            println!("Disk {} has {} partitions", root_dev, partitions.len());
        }
        drives.push(Drive::Local(DiskInfo {
            model,
            total_space,
            partitions,
            is_cdrom,
            icon_name,
        }));
    }
    if verbose {
        println!("Total local disks found: {}", drives.len());
    }

    // Network drives via /proc/mounts
    if let Ok(mounts) = fs::read_to_string("/proc/mounts") {
        for line in mounts.lines() {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() >= 3 {
                let fstype = fields[2];
                if fstype.starts_with("nfs") || matches!(fstype, "cifs" | "smbfs") {
                    let mount_point = fields[1].to_string();
                    match statvfs(mount_point.as_str()) {
                        Ok(stat) => {
                            let block_size = stat.block_size() as u64;
                            let total = stat.blocks() * block_size;
                            let free = stat.blocks_free() * block_size;
                            let used = total - free;

                            if verbose {
                                println!(
                                    "Network drive detected: {} (type: {}, total: {}, used: {})",
                                    mount_point, fstype, total, used
                                );
                            }

                            drives.push(Drive::Network(NetworkDrive {
                                mount_point,
                                used_space: used,
                                total_space: total,
                            }));
                        }
                        Err(e) => {
                            if verbose {
                                println!("Failed to statvfs network drive {}: {:?}", mount_point, e);
                            }
                        }
                    }
                }
            }
        }
    }

    if verbose {
        println!("Total drives returned (local + network): {}", drives.len());
    }
    drives
}