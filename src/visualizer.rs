use super::gif::Frame;
use super::trace::Category;
use super::trace::Event;
use std::borrow::Cow;

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

    pub fn events_to_frame(&self, events: &[Event]) -> Frame {
        let mut buffer = vec![255; self.width as usize * self.height as usize];
        for event in events {
            let buffer_range = self.event_to_range(event);
            let color = self.pick_color(event.category);
            let range = &mut buffer[buffer_range.0..buffer_range.1];
            for i in range.iter_mut() {
                *i = color;
            }
        }
        let mut frame = Frame::default();
        frame.width = self.width;
        frame.height = self.height;
        frame.buffer = Cow::from(buffer);
        frame
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

    fn pick_color(&self, category: Category) -> u8 {
        let mut color: u8 = 255;
        if category.contains(Category::READ) {
            color = 170;
        }
        if category.contains(Category::WRITE) {
            color = 10;
        }
        /*
        if category.contains(Category::FLUSH) {
            color = color | 1 << 2;
        }
        if category.contains(Category::SYNC) {
            color = color | 1 << 3;
        }
        if category.contains(Category::QUEUE) {
            color = color | 1 << 4;
        }
        if category.contains(Category::REQUEUE) {
            color = color | 1 << 5;
        }
        if category.contains(Category::ISSUE) {
            color = color | 1 << 6;
        }
        if category.contains(Category::AHEAD) {
            color = color | 1 << 7;
        }
        */
        color
    }
}
