use crate::disk::Drive;
use crate::progress_bar::ProgressBar;
use crate::sizes::format_size;
use cosmic::iced_widget::button as cosmic_button;
use cosmic::{
    iced::Alignment,
    theme,
    widget::{column, icon, row, text},
    Element,
};

#[derive(Debug, Clone)]
pub struct DiskState {
    pub drives: Vec<Drive>,
    pub expanded: Vec<bool>,
}

impl DiskState {
    pub fn new(drives: Vec<Drive>) -> Self {
        DiskState {
            drives: drives.clone(),
            expanded: vec![false; drives.len()],
        }
    }

    pub fn toggle(&mut self, index: usize) {
        if let Some(expanded) = self.expanded.get_mut(index) {
            *expanded = !*expanded;
        }
    }

    pub fn view(&self) -> Element<Message> {
        let mut disk_tree = column().spacing(8);
        for (i, drive) in self.drives.iter().enumerate() {
            match drive {
                Drive::Local(disk) => {
                    if disk.is_cdrom {
                        let cdrom = &disk.partitions[0];
                        let cdrom_row = cosmic_button(
                            row()
                                .push(icon::from_name(&*disk.icon_name).size(24))
                                .push(
                                    column()
                                        .spacing(2)
                                        .push(text(&disk.model))
                                        .push(
                                            row()
                                                .push(text(&cdrom.device))
                                                .push(text(format!(" - {}", format_size(cdrom.total_space))))
                                                .spacing(4)
                                        )
                                )
                                .spacing(4)
                                .align_y(Alignment::Center),
                        )
                            .on_press(Message::SelectPartition(cdrom.mount_point.clone()))
                            .padding([4, 8]);
                        disk_tree = disk_tree.push(cdrom_row);
                    } else {
                        let disk_row = cosmic_button(
                            row()
                                .push(icon::from_name(&*disk.icon_name).size(24))
                                .push(text(&disk.model))
                                .spacing(4)
                                .align_y(Alignment::Center),
                        )
                            .on_press(Message::ToggleDisk(i))
                            .padding([4, 8]);
                        disk_tree = disk_tree.push(disk_row);

                        if self.expanded.get(i).copied().unwrap_or(false) {
                            let mut partition_list = column().spacing(4);
                            for partition in &disk.partitions {
                                let percent = (partition.used_space as f32 / partition.total_space.max(1) as f32) * 100.0;
                                let is_dark = theme::active().theme_type.is_dark();
                                partition_list = partition_list.push(
                                    cosmic_button(
                                        column()
                                            .spacing(2)
                                            .align_x(Alignment::Center)
                                            .push(text(&partition.device))
                                            .push(ProgressBar::new(is_dark, percent))
                                            .push(text(format!(
                                                "{} / {}",
                                                format_size(partition.used_space),
                                                format_size(partition.total_space)
                                            )))
                                    )
                                        .on_press(Message::SelectPartition(partition.mount_point.clone()))
                                        .padding([6, 8])
                                );
                            }
                            disk_tree = disk_tree.push(partition_list);
                        }
                    }
                }
                Drive::Network(net) => {
                    let percent = (net.used_space as f32 / net.total_space.max(1) as f32) * 100.0;
                    let is_dark = theme::active().theme_type.is_dark();
                    let net_row = cosmic_button(
                        row()
                            .push(icon::from_name("network-workgroup").size(24))
                            .push(
                                column()
                                    .spacing(2)
                                    .align_x(Alignment::Center)
                                    .push(text(&net.mount_point))
                                    .push(ProgressBar::new(is_dark, percent))
                                    .push(text(format!(
                                        "{} / {}",
                                        format_size(net.used_space),
                                        format_size(net.total_space)
                                    )))
                            )
                            .spacing(4)
                            .align_y(Alignment::Center),
                    )
                        .on_press(Message::SelectPartition(net.mount_point.clone()))
                        .padding([4, 8]);
                    disk_tree = disk_tree.push(net_row);
                }
            }
        }
        disk_tree.into()
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleDisk(usize),
    SelectPartition(String),
}