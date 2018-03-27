#[macro_use]
extern crate clap;

extern crate num;
#[macro_use]
extern crate num_derive;
extern crate num_traits;

#[macro_use]
extern crate bitflags;

mod trace;

fn main() {
    use std::fs::File;
    use std::io::Read;
    use clap::{App, Arg};
    use trace::*;

    let matches = App::new("blkview")
        .version(crate_version!())
        .arg(Arg::with_name("files").multiple(true).last(true))
        .get_matches();
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
    println!("Processed {} events", trace.events.len());
    let total_written = trace
        .events
        .iter()
        .filter(|event| {
            event.category.contains(Category::WRITE) && event.action == Action::Complete
        })
        .fold(0, |acc, s| acc + s.bytes);
    println!("Total writes: {}", total_written);
}
