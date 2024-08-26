pub mod obj;
pub mod script;

pub mod replay {
    use std::io::Read;

    use binrw::{binread, BinResult, NullString};
    use serde::{Deserialize, Serialize};

    #[binrw::parser(reader)]
    fn parse_inputs() -> BinResult<Vec<u8>> {
        let mut bytes = Vec::new();

        let mut zlib_decoder = flate2::read::ZlibDecoder::new(reader);
        zlib_decoder.read_to_end(&mut bytes)?;

        Ok(bytes)
    }

    #[binread]
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ReplayTime {
        pub year: u16,
        pub month: u8,
        pub day: u8,
        pub hour: u8,
        pub minute: u8,
        pub second: u8,
    }

    #[binread]
    #[br(repr = u8)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum Character {
        Sol = 1,
        Ky,
        May,
        Millia,
        Axl,
        Potemkin,
        Chipp,
        Eddie,
        Baiken,
        Faust,
        Testament,
        Jam,
        Anji,
        Johnny,
        Venom,
        Dizzy,
        Slayer,
        Ino,
        Zappa,
        Bridget,
        RoboKy,
        Aba,
        OrderSol,
        Kliff,
        Justice,
    }

    #[binread]
    #[br(repr = u8)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum MatchType {
        Single = 1,
        Team,
    }

    #[binread]
    #[br(repr = u8)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum GameVersion {
        PlusR = 0,
        AccentCore,
    }

    #[binread]
    #[br(repr = u8)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum MatchResult {
        P1Winner = 1,
        P2Winner,
        Draw,
    }

    #[binread]
    // MAGIC signature is this literal on all ACPR replays after Dec. 2021
    #[br(little, magic = b"GGR\x02\x51\xAD\xEE\x77\x45\xD7\x48\xCD")]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AcprReplay {
        #[br(temp)]
        _metadata_size: u16,
        #[br(temp)]
        _compressed_input_size: u32,
        #[br(temp)]
        _uncompressed_input_size: u32,

        _replay_hash: u32,
        pub replay_date: ReplayTime,

        _unknown: u8,

        pub p1_steam_id: u64,
        pub p2_steam_id: u64,
        #[br(map = |x: NullString| x.to_string(), pad_size_to = 32)]
        pub p1_name: String,
        #[br(map = |x: NullString| x.to_string(), pad_size_to = 32)]
        pub p2_name: String,
        pub p1_character: Character,
        pub p2_character: Character,

        #[br(map = |x: u8| x != 0)]
        pub special_options_used: bool,
        pub match_type: MatchType,
        pub game_version: GameVersion,
        pub timezone_offset: i32,
        pub p1_rounds_won: u8,
        pub p2_rounds_won: u8,

        // Accessing a simple bitfield without any extra deps
        #[br(restore_position, map = |x: u8| (x & 0x01) != 0)]
        pub match_unfinished: bool,
        #[br(restore_position, map = |x: u8| (x & 0x02) != 0)]
        pub match_disconnected: bool,
        #[br(map = |x: u8| (x & 0x04) != 0)]
        pub match_desynced: bool,

        pub ping: u8,
        pub match_duration: u32,
        pub p1_score: u8,
        pub p2_score: u8,
        pub p1_rank: u8,
        pub p2_rank: u8,
        pub match_result: MatchResult,
        #[br(parse_with = parse_inputs)]
        pub replay_inputs: Vec<u8>,
    }

    crate::helpers::impl_open!(AcprReplay);
}
