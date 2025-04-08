use crate::crawler::FileEntry;
use crate::ui::Message;
use cosmic::iced_renderer::fallback::Renderer;
use cosmic::iced_wgpu::Renderer as WgpuRenderer;
use cosmic::{
    iced_core::{Alignment, Color, Element, Length},
    widget::{column, container, row, text, mouse_area, icon, Icon},
};
use cosmic::iced_widget::scrollable;
use iced_tiny_skia::Renderer as SkiaRenderer;
type CosmicRenderer = Renderer<WgpuRenderer, SkiaRenderer>;

#[derive(Debug, Clone)]
pub struct Files {
    pub mount_point: String,
    pub current_path: String,
    pub entries: Vec<FileEntry>,
    pub verbose: bool,
}

impl Files {
    pub fn load(mount_point: String, current_path: String, verbose: bool) -> Files {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");
        rt.block_on(async {
            let entries = crate::crawler::crawl_files(current_path.clone(), verbose).await;
            if verbose {
                eprintln!("Files loaded entries for {}: {:?}", current_path, entries);
            }
            Files {
                current_path,
                mount_point,
                entries,
                verbose,
            }
        })
    }

    // Calculate the color for a file based on its size
    fn calculate_color(color_value: f32) -> Color {
        if color_value <= 205.0_f32 {
            // Lapis (38, 97, 156) at 0, interpolate to Jade (0, 187, 119) at 205
            let t = color_value / 205.0_f32;
            Color {
                r: (38.0_f32 - t * (38.0_f32 - 0.0_f32)) / 255.0_f32,
                g: (97.0_f32 + t * (187.0_f32 - 97.0_f32)) / 255.0_f32,
                b: (156.0_f32 - t * (156.0_f32 - 119.0_f32)) / 255.0_f32,
                a: 1.0_f32,
            }
        } else if color_value <= 410.0_f32 {
            // Jade (0, 187, 119) at 206, interpolate to Gold (239, 191, 4) at 410
            let t = (color_value - 206.0_f32) / (410.0_f32 - 206.0_f32);
            Color {
                r: (0.0_f32 + t * (239.0_f32 - 0.0_f32)) / 255.0_f32,
                g: (187.0_f32 - t * (187.0_f32 - 191.0_f32)) / 255.0_f32,
                b: (119.0_f32 - t * (119.0_f32 - 4.0_f32)) / 255.0_f32,
                a: 1.0_f32,
            }
        } else if color_value <= 615.0_f32 {
            // Gold (239, 191, 4) at 411, interpolate to Purple (75, 0, 110) at 615
            let t = (color_value - 411.0_f32) / (615.0_f32 - 411.0_f32);
            Color {
                r: (239.0_f32 - t * (239.0_f32 - 75.0_f32)) / 255.0_f32,
                g: (191.0_f32 - t * (191.0_f32 - 0.0_f32)) / 255.0_f32,
                b: (4.0_f32 + t * (110.0_f32 - 4.0_f32)) / 255.0_f32,
                a: 1.0_f32,
            }
        } else if color_value <= 820.0_f32 {
            // Purple (75, 0, 110) at 616, interpolate to Crimson (140, 0, 15) at 820
            let t = (color_value - 616.0_f32) / (820.0_f32 - 616.0_f32);
            Color {
                r: (75.0_f32 + t * (140.0_f32 - 75.0_f32)) / 255.0_f32,
                g: (0.0_f32 + t * (0.0_f32 - 0.0_f32)) / 255.0_f32,
                b: (110.0_f32 - t * (110.0_f32 - 15.0_f32)) / 255.0_f32,
                a: 1.0_f32,
            }
        } else {
            // Crimson (140, 0, 15) at 821, stays Crimson up to 1024
            let t = (color_value - 821.0_f32) / (1024.0_f32 - 821.0_f32);
            Color {
                r: (140.0_f32 + t * (140.0_f32 - 140.0_f32)) / 255.0_f32,
                g: (0.0_f32 + t * (0.0_f32 - 0.0_f32)) / 255.0_f32,
                b: (15.0_f32 + t * (15.0_f32 - 15.0_f32)) / 255.0_f32,
                a: 1.0_f32,
            }
        }
    }

    pub fn view<'a>(&self, _available_height: f32, available_width: f32) -> Element<'a, Message, cosmic::Theme, CosmicRenderer> {
        // Sort entries by size in descending order
        let mut sorted_entries = self.entries.clone();
        sorted_entries.sort_by(|a, b| b.size.cmp(&a.size));

        // Rectangle dimensions
        let rect_width = 50.0_f32;
        let rect_height = 60.0_f32;
        let gap = 1.0_f32;
        let total_rect_width = rect_width + gap;

        // Calculate the number of rectangles that fit per row based on the available width
        let rects_per_row = ((available_width / total_rect_width).round() as usize).saturating_sub(6); // Subtract 6 or it draws off-screen
        let rects_per_row = rects_per_row.max(1);

        // Calculate the number of rows needed
        let _total_rects = sorted_entries.len();

        // Debug print to verify layout calculations, only if verbose is true
        if self.verbose {
            println!(
                "Available width: {}, Total rect width: {}, Rects per row: {}, Buffer: {}",
                available_width,
                total_rect_width,
                rects_per_row,
                available_width - (rects_per_row as f32 * total_rect_width)
            );
        }

        // Create the grid of rectangles
        let mut rows: Vec<Element<'a, Message, cosmic::Theme, CosmicRenderer>> = Vec::new();
        let mut current_row: Vec<Element<'a, Message, cosmic::Theme, CosmicRenderer>> = Vec::new();
        let mut rect_count = 0;

        for entry in sorted_entries.iter() {
            let size = entry.size.max(1) as f32;

            // Map size (in bytes) to 0-1024GB scale (0 to 1 TB)
            let size_in_tb = size / 1_099_511_627_776.0_f32;
            let color_value = (size_in_tb * 1024.0_f32).min(1024.0_f32);
            let color = Self::calculate_color(color_value);

            // Select the icon based on whether the entry is a directory or file
            let icon_name = if entry.is_dir { "folder" } else { "text-x-generic" };
            let icon_widget: Icon = icon::from_name(icon_name)
                .size(48) // Set icon size to 48x48 pixels
                .into();  // Convert Named to Icon

            // Create the rectangle with the icon centered
            let rect = container(icon_widget)
                .width(Length::Fixed(rect_width))
                .height(Length::Fixed(rect_height))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .style(move |_| container::Style {
                    background: Some(color.into()),
                    border: cosmic::iced::Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 12.0.into(),
                    },
                    text_color: None,
                    icon_color: None,
                    shadow: cosmic::iced_core::Shadow::default(),
                });

            let subfolder = entry.path.to_string_lossy().to_string();
            let is_dir = entry.is_dir;
            let rect_with_interaction = mouse_area(rect)
                .on_enter(Message::HoverUpdate(Some(entry.clone())))
                .on_exit(Message::HoverUpdate(None))
                .on_press(if is_dir {
                    Message::CrawlSubfolder(subfolder)
                } else {
                    Message::Click
                });

            // Add gap on the right
            let rect_with_gaps = row()
                .push(rect_with_interaction)
                .push(container(text("")).width(Length::Fixed(gap)));

            current_row.push(rect_with_gaps.into());
            rect_count += 1;

            // If the row is full, add it to the rows list and start a new row
            if rect_count % rects_per_row == 0 {
                let mut row_widget = row().spacing(0.0);
                for element in current_row.drain(..) {
                    row_widget = row_widget.push(element);
                }
                // Constrain the row to the available width to prevent overflow
                rows.push(row_widget
                    .width(Length::Fixed(available_width))
                    .height(Length::Fixed(rect_height))
                    .into());
            }
        }

        // Add the last row if it's not empty
        if !current_row.is_empty() {
            let mut row_widget = row().spacing(0.0);
            for element in current_row.drain(..) {
                row_widget = row_widget.push(element);
            }
            rows.push(row_widget
                .width(Length::Fixed(available_width))
                .height(Length::Fixed(rect_height))
                .into());
        }

        // Create a scrollable column of rows
        let mut column_widget = column().spacing(1.0);
        for row in rows {
            column_widget = column_widget.push(row);
        }
        scrollable(column_widget)
            .width(Length::Fixed(available_width))
            .height(Length::Fill)
            .into()
    }
}