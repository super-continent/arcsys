//! Types that allow easy parsing and rebuilding of various Arc System Works file formats.

mod error;
mod file_traits;
mod helpers;

pub mod bbcf;
pub mod ggst;

pub use error::Error;

#[cfg(test)]
mod tests {
    use std::fs::File;


    // This requires you to place your own pac files taken from the games into a test_files folder
    static GGST_PAC: &[u8] = include_bytes!("../test_files/ggst.pac");
    #[test]
    fn test_pac() {
        use crate::bbcf;
        use crate::ggst;

        
        let bbcf_pac = include_bytes!("../test_files/bbcf.pac");

        let ggst_parsed = ggst::pac::GGSTPac::parse(GGST_PAC);
        let bbcf_parsed = bbcf::pac::BBCFPac::parse(bbcf_pac);

        // Ensure pacs parse correctly
        assert!(ggst_parsed.is_ok());
        assert!(bbcf_parsed.is_ok());

        // Ensure pacs fail to parse when whole file is not consumed or other error occurs
        assert!(bbcf::pac::BBCFPac::parse(GGST_PAC).is_err());
        assert!(ggst::pac::GGSTPac::parse(bbcf_pac).is_err());

        // test rebuilding
        let bbcf_bytes = bbcf_parsed.unwrap().to_bytes();
        assert_eq!(bbcf_bytes, bbcf_pac);

        let ggst_bytes = ggst_parsed.unwrap().to_bytes();
        assert_eq!(ggst_bytes, GGST_PAC);
    }

    #[test]
    fn test_jonbin() {
        use crate::ggst;

        let parsed = ggst::pac::GGSTPac::parse(GGST_PAC).unwrap();

        // Iterate through all the jonbins contained in the pac
        for entry in parsed.files {
            eprintln!("parsing: {}", entry.name);
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

        let hip = include_bytes!("../test_files/bbcf.hip");

        //let parsed_hip = bbcf::hip::BBCFHip::parse(hip);
    }
}
