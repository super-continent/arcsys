//! Types that allow easy parsing and rebuilding of various Arc System Works file formats.
#![feature(seek_stream_len)]

mod error;
mod helpers;
mod traits;

pub use binrw::BinRead;

pub use crate::traits::{ParseFromBytes, Rebuild};

/// Blazblue Centralfiction
pub mod bbcf;
/// Guilty Gear XX Accent Core +R
pub mod ggacpr;
/// Guilty Gear STRIVE
pub mod ggst;
/// PAC archive format found in most arcsys games.
pub mod pac;

pub use error::Error;
pub use helpers::{arcsys_filename_hash, IndexedImage, RGBAColor};

#[cfg(test)]
mod tests {
    use std::env;
    use std::io::{Cursor, Read};
    use std::path::PathBuf;

    use binrw::BinRead;
    use walkdir::WalkDir;

    #[test]
    fn test_pac() {
        let pac_path = PathBuf::from(
            env::var("ARCSYS_PACS").expect("Must have ARCSYS_PACS env variable set!"),
        );
        let paths = WalkDir::new(pac_path);

        for entry in paths.into_iter().map(|x| x.unwrap()) {
            let path = entry.path();

            // skip TXAC
            if path.to_string_lossy().contains("txac") {
                println!("skipping TXAC: {path:?}");
                continue;
            }

            if path.is_dir() {
                continue;
            }

            println!("parsing: {path:?}");

            let mut pac_bytes = Vec::new();
            std::fs::File::open(path)
                .unwrap()
                .read_to_end(&mut pac_bytes)
                .unwrap();

            let pac = crate::pac::Pac::read(&mut Cursor::new(&pac_bytes)).unwrap();

            println!("info: {:?}", pac.pac_style);

            let rebuilt_bytes = pac.to_bytes();

            if rebuilt_bytes != pac_bytes {
                let file_name = path.file_stem().unwrap().to_string_lossy();

                let _ = std::fs::create_dir("TEST_OUTPUT");

                std::fs::write(
                    format!("./TEST_OUTPUT/{file_name}_REBUILT.pac"),
                    rebuilt_bytes,
                )
                .unwrap();
                std::fs::write(format!("./TEST_OUTPUT/{file_name}_ORIGINAL.pac"), pac_bytes)
                    .unwrap();

                // if file_name.contains("bg_halloween_4") {
                //     pac.entries.iter().for_each(|x| println!("{:?}", x.name))
                // }
                panic!("rebuilt bytes dont match for {}", path.display());
            }
        }
    }
}
