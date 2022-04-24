//! Types that allow easy parsing and rebuilding of various Arc System Works file formats.

mod error;
mod helpers;
mod traits;

/// Blazblue Centralfiction
pub mod bbcf;
/// Guilty Gear STRIVE
pub mod ggst;
/// Guilty Gear XX Accent Core +R
pub mod ggacpr;

pub use error::Error;
pub use helpers::{IndexedImage, RGBAColor};

#[cfg(test)]
mod tests {
    //use std::io::Write;

    // This requires you to place your own pac files taken from the games into a test_files folder

    use binrw::BinRead;

    // PAC files that contain jonbin (collision) files
    static GGST_JONBINS_PAC: &[u8] = include_bytes!("../test_files/ggst_jonbins.pac");
    static BBCF_JONBINS_PAC: &[u8] = include_bytes!("../test_files/bbcf_jonbins.pac");

    // PAC files that contain HIP (image) files
    static BBCF_COMPRESSED_HIPS_PAC: &[u8] = include_bytes!("../test_files/char_jb_img.pac");

    static ACPR_REPLAY: &[u8] = include_bytes!("../test_files/test_replay.ggr");

    #[test]
    fn test_pac() {
        use crate::bbcf;
        use crate::ggst;

        let ggst_parsed = ggst::pac::GGSTPac::parse(GGST_JONBINS_PAC);
        let bbcf_parsed = bbcf::pac::BBCFPac::parse(BBCF_JONBINS_PAC);

        // Ensure pacs parse correctly
        assert!(ggst_parsed.is_ok());
        assert!(bbcf_parsed.is_ok());

        // Ensure pacs fail to parse when whole file is not consumed or other error occurs
        assert!(bbcf::pac::BBCFPac::parse(GGST_JONBINS_PAC).is_err());
        assert!(ggst::pac::GGSTPac::parse(BBCF_JONBINS_PAC).is_err());

        // test rebuilding
        let bbcf_bytes = bbcf_parsed.unwrap().to_bytes();
        assert_eq!(bbcf_bytes, BBCF_JONBINS_PAC);

        let ggst_bytes = ggst_parsed.unwrap().to_bytes();
        assert_eq!(ggst_bytes, GGST_JONBINS_PAC);

        // re-encode a characters sprites, for testing possible visual information corruption
        //     let mut bbcf_parsed_hips = bbcf::pac::BBCFPac::parse(BBCF_COMPRESSED_HIPS_PAC).unwrap();

        //     for sprite in &mut bbcf_parsed_hips.files {
        //         let reencoded = bbcf::hip::BBCFHip::parse(&sprite.contents).unwrap().to_bytes();

        //         sprite.contents = reencoded;
        //     }

        //     std::fs::File::create("./compressed_bytes.pac").unwrap().write(&bbcf_parsed_hips.to_bytes_compressed());
    }

    #[test]
    fn test_jonbin() {
        use crate::ggst;

        let parsed = ggst::pac::GGSTPac::parse(GGST_JONBINS_PAC).unwrap();

        // Iterate through all the jonbins contained in the pac
        for entry in parsed.files {
            //eprintln!("parsing: {}", entry.name);
            let parsed_jonbin = ggst::jonbin::GGSTJonBin::parse(&entry.contents);

            if let Err(ref e) = parsed_jonbin {
                println!("{}", e);
            }

            assert!(parsed_jonbin.is_ok());
            let bytes = parsed_jonbin.unwrap().to_bytes();
            assert_eq!(entry.contents.to_vec(), bytes);
        }
    }

    #[test]
    fn test_hip() {
        use crate::bbcf;

        // HIP file using raw ARGB
        let raw_image_hip = include_bytes!("../test_files/bbcf_raw_image.hip");

        let parsed = bbcf::pac::BBCFPac::parse(BBCF_COMPRESSED_HIPS_PAC).unwrap();

        let parsed_hip = bbcf::hip::BBCFHip::parse(raw_image_hip);
        if let Err(ref e) = parsed_hip {
            eprintln!("{}", e);
        }
        assert!(parsed_hip.is_ok());

        // Iterate through all the hips contained in the pac
        for entry in &parsed.files {
            //eprintln!("parsing: {}", entry.name);
            let parsed_hip = bbcf::hip::BBCFHip::parse(&entry.contents);

            if let Err(ref e) = parsed_hip {
                dbg!(&entry.name);
                eprintln!("{}", e);
            }

            assert!(parsed_hip.is_ok());
        }

        //let palette_image = &parsed.files[0];

        //let hip = bbcf::hip::BBCFHip::parse(&palette_image.contents).unwrap();
        //let bytes = hip.to_bytes();
    }

    #[test]
    fn test_acpr_replay() {
        use crate::ggacpr::replay::AcprReplay;
        let bytes: Vec<u8> = ACPR_REPLAY.to_vec();


        let replay_data = AcprReplay::read(&mut std::io::Cursor::new(bytes));

        assert!(replay_data.is_ok());
    }
}
