use crate::files::Files;
use crate::disk::{scan_disks, Drive};
use crate::partition::{DiskState, Message as PartitionMessage};
use cosmic::iced_core::{Border, Element, Shadow, Point};
use cosmic::iced_renderer::fallback::Renderer;
use cosmic::iced_wgpu::Renderer as WgpuRenderer;
use cosmic::{app::{Application, Core, Task},
             iced::widget::container::Style,
             iced::{Alignment, Color, Length, Subscription, Event, mouse},
             widget::{column, container, icon, row, scrollable, text, mouse_area}, Apply};
use cosmic::iced_widget::button;
use iced_tiny_skia::Renderer as SkiaRenderer;
use crate::sizes::format_size;

use cosmic::iced::window::Event as WindowEvent;

fn panel_style(theme: &cosmic::Theme) -> Style {
    Style {
        border: Border {
            color: Color::from(theme.cosmic().accent_button.base),
            width: 1.0,
            radius: 12.0.into(),
        },
        shadow: Shadow {
            color: Color { r: 0.0, g: 0.0, b: 0.0, a: 0.15 },
            offset: [0.0, 1.0].into(),
            blur_radius: 3.0,
        },
        ..Default::default()
    }
}

#[derive(Debug, Clone)]
enum FilesState {
    None,
    Loading(String),
    Ready(Files, Option<crate::crawler::FileEntry>),
}

pub struct CosmicDust {
    core: Core,
    total_space: u64,
    used_space: u64,
    disk_state: DiskState,
    files_state: FilesState,
    verbose: bool,
    cursor_position: Point,
    window_size: cosmic::iced::Size,
}

#[derive(Debug, Clone)]
pub enum Message {
    ScanUpdate(Vec<Drive>),
    Disk(PartitionMessage),
    FilesLoaded(Files),
    CrawlSubfolder(String),
    HoverUpdate(Option<crate::crawler::FileEntry>),
    UpButtonClicked,
    Refresh,
    CursorMoved(Point),
    Click,
    WindowResized(cosmic::iced::Size),
}

impl Application for CosmicDust {
    type Executor = cosmic::iced::executor::Default;
    type Flags = bool;
    type Message = Message;

    const APP_ID: &'static str = "io.melechtna.CosmicDust";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, verbose: Self::Flags) -> (Self, Task<Self::Message>) {
        let scan_task = Task::perform(scan_disks(), |drives| cosmic::Action::App(Message::ScanUpdate(drives)));

        let default_size = cosmic::iced::Size { width: 1280.0, height: 720.0 };

        (
            Self {
                core,
                total_space: 0,
                used_space: 0,
                disk_state: DiskState::new(Vec::new()),
                files_state: FilesState::None,
                verbose,
                cursor_position: Point::new(0.0, 0.0),
                window_size: default_size,
            },
            scan_task,
        )
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        cosmic::iced::event::listen_raw(|event, _status, _context| {
            match event {
                Event::Mouse(mouse::Event::CursorMoved { position }) => Some(Message::CursorMoved(position)),
                Event::Window(WindowEvent::Resized(size)) => {
                    Some(Message::WindowResized(size))
                }
                _ => None,
            }
        })
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::ScanUpdate(drives) => {
                self.total_space = drives.iter().map(|d| match d {
                    Drive::Local(disk) => disk.total_space,
                    Drive::Network(net) => net.total_space,
                }).sum();
                self.used_space = drives.iter().map(|d| match d {
                    Drive::Local(disk) => disk.partitions.iter().map(|p| p.used_space).sum::<u64>(),
                    Drive::Network(net) => net.used_space,
                }).sum();
                self.disk_state = DiskState::new(drives);
                self.files_state = FilesState::None;
                Task::none()
            }
            Message::Disk(PartitionMessage::ToggleDisk(index)) => {
                self.disk_state.toggle(index);
                Task::none()
            }
            Message::Disk(PartitionMessage::SelectPartition(mount)) => {
                self.files_state = FilesState::Loading(mount.clone());
                let verbose = self.verbose;
                Task::perform(
                    async move { Files::load(mount.clone(), mount, verbose) },
                    |files| cosmic::Action::App(Message::FilesLoaded(files)),
                )
            }
            Message::FilesLoaded(files) => {
                self.files_state = FilesState::Ready(files, None);
                Task::none()
            }
            Message::CrawlSubfolder(subfolder) => {
                let mount_point = if let FilesState::Ready(files, _) = &self.files_state {
                    files.mount_point.clone()
                } else {
                    subfolder.clone()
                };

                self.files_state = FilesState::Loading(subfolder.clone());
                let verbose = self.verbose;
                Task::perform(
                    async move { Files::load(mount_point, subfolder, verbose) },
                    |files| cosmic::Action::App(Message::FilesLoaded(files)),
                )
            }
            Message::HoverUpdate(hovered) => {
                if let FilesState::Ready(_, ref mut current_hovered) = &mut self.files_state {
                    *current_hovered = hovered;
                }
                Task::none()
            }
            Message::UpButtonClicked => {
                let (current_path, mount_point) = if let FilesState::Ready(files, _) = &self.files_state {
                    (files.current_path.clone(), files.mount_point.clone())
                } else {
                    return Task::none();
                };

                if current_path == mount_point {
                    return Task::none();
                }

                let parent_path = match std::path::Path::new(&current_path).parent() {
                    Some(parent) => parent.to_string_lossy().to_string(),
                    None => mount_point.clone(),
                };

                let parent_path = if parent_path.is_empty() {
                    mount_point.clone()
                } else {
                    parent_path
                };

                if parent_path == current_path {
                    return Task::none();
                }

                self.files_state = FilesState::Loading(mount_point.clone());
                let verbose = self.verbose;
                Task::perform(
                    async move { Files::load(mount_point, parent_path, verbose) },
                    |files| cosmic::Action::App(Message::FilesLoaded(files)),
                )
            }
            Message::Refresh => Task::perform(scan_disks(), |drives| cosmic::Action::App(Message::ScanUpdate(drives))),
            Message::CursorMoved(position) => {
                self.cursor_position = position;
                Task::none()
            }
            Message::Click => {
                Task::none()
            }
            Message::WindowResized(size) => {
                self.window_size = size;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Self::Message, cosmic::Theme, Renderer<WgpuRenderer, SkiaRenderer>> {
        let left_panel = container(
            column()
                .push(scrollable(self.disk_state.view().map(Message::Disk)).width(Length::Shrink).height(Length::Fill))
                .push(
                    row()
                        .push(button(
                            row()
                                .push(icon::from_name("view-refresh").size(24))
                                .push(text("Refresh"))
                                .spacing(4)
                                .align_y(Alignment::Center),
                        ).on_press(Message::Refresh))
                        .spacing(4)
                        .padding([4, 0])
                        .width(Length::Shrink)
                )
                .spacing(8)
                .width(Length::Shrink),
        )
            .style(panel_style)
            .padding(8)
            .width(Length::Shrink);

        let right_panel_content = match &self.files_state {
            FilesState::None => container(
                column()
                    .push(text("Please select partition"))
                    .align_x(Alignment::Center)
                    .width(Length::Fill)
            )
                .align_y(Alignment::Center)
                .height(Length::Fill),
            FilesState::Loading(mount) => container(
                column()
                    .spacing(8)
                    .push(
                        text(self.disk_state.drives.iter().find_map(|drive| match drive {
                            Drive::Local(disk) if !disk.is_cdrom => disk.partitions.iter()
                                .find(|p| &p.mount_point == mount)
                                .map(|p| format!("{} - ({})", mount, p.file_system)),
                            _ => Some(mount.clone()),
                        }).unwrap_or(mount.clone()))
                            .align_x(Alignment::Center)
                    )
                    .push(
                        container(
                            column()
                                .push(text("Loading..."))
                                .align_x(Alignment::Center)
                                .width(Length::Fill)
                        )
                            .align_y(Alignment::Center)
                            .height(Length::Fill)
                    )
            ),
            FilesState::Ready(files, hovered) => {
                let up_button: Element<Self::Message, cosmic::Theme, Renderer<WgpuRenderer, SkiaRenderer>> = if files.current_path != files.mount_point {
                    button(
                        text("^")
                            .size(18)
                            .font(cosmic::font::bold())
                            .align_x(Alignment::Center)
                            .align_y(Alignment::Center)
                    )
                        .width(Length::Fixed(30.0))
                        .height(Length::Fixed(30.0))
                        .on_press(Message::UpButtonClicked)
                        .into()
                } else {
                    container(text(""))
                        .width(Length::Fixed(30.0))
                        .into()
                };

                let top_row = row()
                    .push(
                        text(&files.current_path)
                            .size(16.0)
                            .width(Length::Fill)
                            .align_y(Alignment::Center)
                    )
                    .push(up_button)
                    .spacing(8)
                    .height(Length::Fixed(30.0));

                let window_height = self.window_size.height;
                let window_width = self.window_size.width;
                let top_row_height = 30.0;
                let bottom_row_height = 30.0;
                let column_spacing = 8.0;
                let padding = 8.0 * 2.0;
                let available_height = window_height - top_row_height - bottom_row_height - column_spacing - padding;
                let available_width = window_width - padding;

                let files_area = container(files.view(available_height, available_width))
                    .width(Length::Fill)
                    .height(Length::Fill);

                let bottom_row = container(
                    text(
                        hovered.as_ref().map_or(String::new(), |entry| {
                            let file_name = entry.path.file_name().unwrap_or_default().to_string_lossy();
                            format!("{} ({})", file_name, format_size(entry.size))
                        })
                    )
                        .size(16.0)
                        .align_x(Alignment::Center)
                        .width(Length::Fill)
                )
                    .height(Length::Fixed(30.0));

                container(
                    column()
                        .push(top_row)
                        .push(files_area)
                        .push(bottom_row)
                        .spacing(8)
                        .width(Length::Fill)
                        .height(Length::Fill)
                )
            }
        };

        let right_panel = container(
            mouse_area(right_panel_content)
                .on_press(Message::Click)
        )
            .style(panel_style)
            .padding(8)
            .width(Length::FillPortion(2))
            .height(Length::Fill);

        row()
            .push(left_panel)
            .push(right_panel)
            .spacing(8)
            .width(Length::Fill)
            .height(Length::Fill)
            .apply(container)
            .padding(8)
            .into()
    }
}