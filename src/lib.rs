//! Types that allow easy parsing and rebuilding of various Arc System Works file formats.

mod error;
mod helpers;
mod traits;

pub use binrw::BinRead;

pub use crate::traits::{ParseFromBytes, Rebuild};

/// Blazblue Centralfiction
pub mod bbcf;
/// Dragon Ball Z: Extreme Butoden and One Piece: Great Pirate Colosseum
pub mod dbzop;
/// Guilty Gear XX Accent Core +R
pub mod ggacpr;
/// Guilty Gear STRIVE
pub mod ggst;

pub use error::Error;
pub use helpers::{arcsys_filename_hash, IndexedImage, RGBAColor};

#[cfg(test)]
mod tests {

}
