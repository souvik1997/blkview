#[macro_use]
extern crate clap;

extern crate num;
#[macro_use]
extern crate num_derive;
extern crate num_traits;

#[macro_use]
extern crate bitflags;

extern crate gif;

extern crate rayon;

mod trace;
mod visualizer;
use std::path::{Path, PathBuf};
use trace::*;

fn main() {
    use clap::{App, Arg};
    use std::fs::File;
    use std::io::Read;
    use trace::*;

    let matches = App::new("blkview")
        .version(crate_version!())
        .arg(Arg::with_name("files").multiple(true).last(true))
        .arg(
            Arg::with_name("chunksize")
                .short("c")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .required(true)
                .takes_value(true),
        )
        .get_matches();
    let output = PathBuf::from(matches.value_of("output").unwrap());
    let chunksize: usize = matches
        .value_of("chunksize")
        .unwrap()
        .parse()
        .expect("failed to parse number");
    let files = matches
        .values_of("files")
        .map(|vals| vals.flat_map(|val| File::open(val)).collect::<Vec<_>>())
        .expect("No input files");
    let trace = Trace::new(
        files
            .into_iter()
            .map(|mut file| {
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)
                    .expect("failed to read from file");
                buf
            })
            .collect::<Vec<_>>(),
    );
    println!("Found {} events", trace.events.len());
    let complete_events = trace
        .events
        .into_iter()
        .filter(|e| {
            e.action == Action::Complete && e.category.intersects(Category::READ | Category::WRITE)
        })
        .collect::<Vec<_>>();
    generate_gif(&complete_events, chunksize, &output);
}

fn generate_gif(events: &[Event], chunksize: usize, output: &Path) {
    use gif::{Encoder, Frame, SetParameter};
    use rayon::prelude::*;
    use std::fs::File;
    use visualizer::*;
    let min_sector = events.iter().map(|event| event.sector).min().unwrap();
    let max_sector = events
        .iter()
        .map(|event| event.ending_sector())
        .max()
        .unwrap();
    let visualizer = Visualizer::new(min_sector, max_sector, 200, 200);
    let mut image = File::create(output).expect("failed to create file");
    let mut color_palette = [0 as u8; 256 * 3];
    for x in 0..256 {
        let z = (x & 0b11000000) >> 4;
        let r = x & 0b00000011;
        let b = (x & 0b00001100) >> 2;
        let g = (x & 0b00110000) >> 4;
        color_palette[3 * x] = (r + z) as u8 * 16;
        color_palette[3 * x + 1] = (g + z) as u8 * 16;
        color_palette[3 * x + 2] = (b + z) as u8 * 16;
    }

    let mut encoder = Encoder::new(
        &mut image,
        visualizer.width,
        visualizer.height,
        &color_palette,
    ).unwrap();
    encoder.set(gif::Repeat::Infinite).unwrap();
    let chunks: Vec<&[Event]> = events.chunks(chunksize).collect();
    let frames: Vec<Frame> = chunks
        .par_iter()
        .map(|chunk| visualizer.events_to_frame(chunk))
        .collect();
    println!("Generated {} gif frames", frames.len());
    for frame in &frames {
        encoder.write_frame(frame).unwrap()
    }
}
