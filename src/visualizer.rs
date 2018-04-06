use super::gif::Frame;
use super::trace::Event;

pub struct Visualizer {
    start_sector: u64,
    end_sector: u64,
    pub width: u16,
    pub height: u16,
}

impl Visualizer {
    pub fn new(start_sector: u64, end_sector: u64, width: u16, height: u16) -> Self {
        Visualizer {
            start_sector: start_sector,
            end_sector: end_sector,
            width: width,
            height: height,
        }
    }

    pub fn events_to_heatmap_frame(&self, events: &[Event]) -> Frame {
        let mut buffer = vec![0 as usize; self.width as usize * self.height as usize];
        for event in events {
            let buffer_range = self.event_to_range(event);
            let range = &mut buffer[buffer_range.0..buffer_range.1];
            for i in range.iter_mut() {
                *i += 1;
            }
        }
        let max_freq: usize = *buffer.iter().max().unwrap_or(&2);
        let rgb = buffer.iter().fold(Vec::new(), |mut acc, b| {
            let color = self.frequency_to_color(*b, 0, max_freq);
            acc.push(color.0);
            acc.push(color.1);
            acc.push(color.2);
            acc
        });
        Frame::from_rgb(self.width, self.height, &rgb)
    }

    fn frequency_to_color(&self, frequency: usize, min_freq: usize, max_freq: usize) -> (u8, u8, u8) {
        use super::palette::{Hsl, LinSrgb, RgbHue, pixel::RgbPixel};
        let hsl = {
            if frequency == min_freq || min_freq == max_freq {
                Hsl::new(RgbHue::from(-180.0), 1.0, 1.0)
            } else {
                Hsl::new(RgbHue::from((((frequency - min_freq) as f64).log2() / ((max_freq - min_freq) as f64).log2()) * 180.0 + 180.0), 1.0, 0.50)
            }
        };
        let rgba: (f64, f64, f64, f64) = LinSrgb::from(hsl).into_pixel::<(f64, f64, f64)>().to_rgba();
        ((rgba.0 * 255 as f64) as u8, (rgba.1 * 255 as f64) as u8, (rgba.2 * 255 as f64) as u8)
    }

    fn event_to_range(&self, event: &Event) -> (usize, usize) {
        use std::cmp::max;
        let total_sectors = self.end_sector - self.start_sector;
        let max_index: usize = self.width as usize * self.height as usize - 1;
        let start_index = ((event.sector as f64 - self.start_sector as f64) / (total_sectors as f64)
            * (max_index as f64))
            .floor() as usize;
        let end_sector = max(event.ending_sector(), event.sector + 1);
        let end_index = ((end_sector as f64 - self.start_sector as f64) / (total_sectors as f64)
            * (max_index as f64))
            .floor() as usize;
        (start_index, max(end_index, start_index + 1))
    }
}
