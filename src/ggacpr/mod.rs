pub mod replay {
    use binrw::{binread, NullString, BinRead};

    use crate::Error;

    #[binread]
    // unsure what the data after GGR means,
    // but it seems to remain consistent across replays
    #[br(little, magic = b"GGR")]
    #[derive(Clone)]
    pub struct AcprReplay {
        _signature: [u8; 9],
        _metadata_size: u16,
        _unknown: [u8; 12],

        pub replay_date: ReplayTime,

        _unknown2: u8,

        pub p1_steam_id: u64,
        pub p2_steam_id: u64,
        #[br(pad_size_to = 32)]
        pub p1_name: NullString,
        #[br(pad_size_to = 32)]
        pub p2_name: NullString,
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
    }

    impl AcprReplay {
        pub fn parse<R: std::io::Read + std::io::Seek>(reader: &mut R) -> Result<Self, Error> {
            AcprReplay::read(reader).map_err(|e| Error::Parser(e.to_string()))
        }
    }

    #[binread]
    #[derive(Debug, Clone)]
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
    #[derive(Debug, Clone, Copy)]
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
    #[derive(Debug, Clone, Copy)]
    pub enum MatchType {
        Single = 1,
        Team,
    }

    #[binread]
    #[br(repr = u8)]
    #[derive(Debug, Clone, Copy)]
    pub enum GameVersion {
        PlusR = 0,
        AccentCore,
    }

    #[binread]
    #[br(repr = u8)]
    #[derive(Debug, Clone, Copy)]
    pub enum MatchResult {
        P1Winner = 1,
        P2Winner,
        Draw,
    }
}
