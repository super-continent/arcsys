use std::{fs, io::Write, path::{Path, PathBuf}};

use anyhow::Result as AResult;
use arcsys::{arcsys_filename_hash, ggacpr::replay::AcprReplay};
use clap::{Args, Parser, Subcommand};
use smallvec::SmallVec;

fn main() -> AResult<()> {
    let res = run();
    if let Err(ref e) = res {
        println!("{e}");
    };

    res
}

/// A CLI tool for parsing and rebuilding
/// Arc System Works file formats.
#[derive(Parser, Debug)]
struct Cmd {
    #[clap(subcommand)]
    subcmd: Type,
}

#[derive(Subcommand, Debug)]
enum Type {
    /// PAC Archive formats
    Archive {
        #[command(subcommand)]
        format: PacType,
    },
    /// Guilty Gear XX
    Bin {
        #[command(subcommand)]
        format: BinType,
    },
    /// Guilty Gear XX Accent Core +R
    Acpr {
        #[command(subcommand)]
        format: AcprType,
    },
    /// General utilities for things like debugging
    Utils {
        #[command(subcommand)]
        util: Utility,
    },
}

#[derive(Subcommand, Debug)]
enum Utility {
    /// Hash bytes (e.g. `0xABCDABCD`) using the arcsys filename hash function
    Hash { bytes: String },
}

#[derive(Subcommand, Debug)]
enum PacType {
    Pac {
        #[command(subcommand)]
        action: FileAction,
    },
    Zcmp {
        #[command(subcommand)]
        action: FileAction,
    },
    Dfaspac {
        #[command(subcommand)]
        action: FileAction,
    },
}

#[derive(Subcommand, Debug)]
enum BinType {
    Obj {
        #[command(subcommand)]
        action: FileAction,
    },
}

#[derive(Subcommand, Debug)]
enum AcprType {
    Ggr {
        #[command(subcommand)]
        action: FileAction,
    },
}

#[derive(Parser, Debug)]
enum FileAction {
    /// Parse this format for easier modification
    Parse {
        #[clap(flatten)]
        args: FileActionArgs,
    },
    /// Rebuild file(s) into this format
    Rebuild {
        #[clap(flatten)]
        args: FileActionArgs,
    },
}

#[derive(Args, Debug)]
struct FileActionArgs {
    /// The file or folder to input
    file_in: PathBuf,
    /// Allow overwriting files that already exist
    #[clap(short, long)]
    overwrite: bool,
    /// Path where modified file or folder should be output
    file_out: Option<PathBuf>,
}

fn run() -> AResult<()> {
    let args = Cmd::parse();
    match args.subcmd {
        Type::Archive { format } => match format {
            PacType::Pac { action } => match action {
                FileAction::Parse { args } => parse_pac(args),
                FileAction::Rebuild { args: _ } => todo!(),
            },
            PacType::Zcmp { action } => match action {
                FileAction::Parse { args } => parse_zcmp(args),
                FileAction::Rebuild { args: _ } => todo!(),
            },
            PacType::Dfaspac { action } => match action {
                FileAction::Parse { args } => parse_dfasfpac(args),
                FileAction::Rebuild { args: _ } => todo!(),
            },
        },
        Type::Bin { format } => match format {
            BinType::Obj { action } => match action {
                FileAction::Parse { args } => parse_obj(args),
                FileAction::Rebuild { args: _ } => {
                    unimplemented!("Object bin rebuilding isn't implemented.")
                }
            }
        }
        Type::Acpr { format } => match format {
            AcprType::Ggr { action } => match action {
                FileAction::Parse { args } => parse_ggr(args),
                FileAction::Rebuild { args: _ } => {
                    unimplemented!("ACPR replay files can only be constructed by the executable")
                }
            },
        },
        Type::Utils { util } => match util {
            Utility::Hash { bytes } => {
                let bytes = bytes.trim_start_matches("0x").replace(" ", "");
                if bytes.len() % 2 != 0 || !bytes.chars().all(|x| x.is_ascii_hexdigit()) {
                    return Err(anyhow::anyhow!("Invalid bytes"));
                }

                let bytes = hex::decode(bytes)?;
                println!(
                    "ArcSys filename hash of data: 0x{:X}",
                    arcsys::arcsys_filename_hash(&bytes)
                );

                Ok(())
            }
        },
    }
}

use std::fs::File;
use arcsys::ggacpr::obj::GGXXObjBin;

fn parse_pac(args: FileActionArgs) -> AResult<()> {
    let pac = arcsys::pac::Pac::open(args.file_in)?;

    println!("{:X}: {:?}", pac.pac_style.bits(), pac.pac_style);

    Ok(())
}

fn parse_zcmp(args: FileActionArgs) -> AResult<()> {
    let pac = arcsys::pac::Zcmp::open(&args.file_in)?;

    if let Some(out_path) = args.file_out {
        write_file(out_path, args.overwrite, &pac.data)?;
    }

    Ok(())
}

fn parse_dfasfpac(args: FileActionArgs) -> AResult<()> {
    let pac = arcsys::pac::DfasFPac::open(args.file_in)?;

    if let Some(out_path) = args.file_out {
        write_file(out_path, args.overwrite, &pac.data)?;
    }

    Ok(())
}

fn parse_obj(args: FileActionArgs) -> AResult<()> {
    let obj = GGXXObjBin::open(args.file_in)?;

    if let Some(out_path) = args.file_out {
        for (j, cell) in obj.player.cell_array.cell_entries.iter().enumerate() {
            write_file(
                out_path.join(format!("player/cells/cell_{}.json", j)),
                args.overwrite,
                serde_json::to_string_pretty(&cell)?.as_bytes(),
            )?;
        }
        for (j, sprite) in obj.player.sprite_array.sprite_entries.iter().enumerate() {
            let vec32 = &sprite.data;

            let byte_array = unsafe { vec32.align_to::<u8>().1 };

            write_file(
                out_path.join(format!("player/sprites/sprite_{}.bin", j)),
                args.overwrite,
                byte_array,
            )?;
        }

        write_file(
            out_path.join("player/script.json".to_string()),
            args.overwrite,
            serde_json::to_string_pretty(&obj.player.script_data)?.as_bytes(),
        )?;

        for (j, palette) in obj.player.palette_array.palette_entries.iter().enumerate() {
            write_file(
                out_path.join(format!("palette/pal_{}.bin", j)),
                args.overwrite,
                bincode::serialize(&palette).unwrap(),
            )?;
        }

        for (i, game_object) in obj.objects.iter().enumerate() {
            for (j, cell) in game_object.cell_array.cell_entries.iter().enumerate() {
                write_file(
                    out_path.join(format!("objno{}/cells/cell_{}.json", i, j)),
                    args.overwrite,
                    serde_json::to_string_pretty(&cell)?.as_bytes(),
                )?;
            }
            for (j, sprite) in game_object.sprite_array.sprite_entries.iter().enumerate() {
                let vec32 = &sprite.data;

                let byte_array = unsafe { vec32.align_to::<u8>().1 };

                write_file(
                    out_path.join(format!("objno{}/sprites/sprite_{}.bin", i, j)),
                    args.overwrite,
                    byte_array,
                )?;
            }

            write_file(
                out_path.join(format!("objno{}/script.json", i)),
                args.overwrite,
                serde_json::to_string_pretty(&game_object.script_data)?.as_bytes(),
            )?;
        }

        write_file(
            out_path.join(format!("player/unk_section.bin")),
            args.overwrite,
            obj.unk_section,
        )?;
    }

    Ok(())
}

fn parse_ggr(args: FileActionArgs) -> AResult<()> {
    let ggr = AcprReplay::open(args.file_in)?;

    if let Some(out_path) = args.file_out {
        write_file(
            out_path,
            args.overwrite,
            serde_json::to_string_pretty(&ggr)?.as_bytes(),
        )?;
    }

    Ok(())
}

fn write_file(out_path: impl AsRef<Path>, overwrite: bool, data: impl AsRef<[u8]>) -> AResult<()> {
    let out_path = out_path.as_ref();

    if out_path.exists() && !overwrite {
        return Err(anyhow::anyhow!(
            "Output file or folder already exists! Enable overwriting with -o"
        ));
    }

    fs::create_dir_all(out_path.parent().unwrap())?;
    File::create(out_path)?.write_all(data.as_ref())?;

    Ok(())
}

#[allow(dead_code)]
fn bruteforce_name(hash: u32) -> String {
    use rayon::prelude::*;

    let generator = SequenceGenerator {
        alphabet: &[
            'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q',
            'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '1', '2', '3', '4', '5', '6', '7', '8',
            '9', '0', '_', '.', '/',
        ],
        raw_string: SmallVec::new(),
    };

    let name = generator
        .par_bridge()
        .find_first(|s| hash == arcsys_filename_hash(s))
        .unwrap();

    name
}

struct SequenceGenerator {
    alphabet: &'static [char],
    raw_string: SmallVec<[usize; 64]>,
}

impl Iterator for SequenceGenerator {
    type Item = String;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.raw_string.len() > 64 {
            return None;
        }

        let mut add_digit = true;
        for i in &mut self.raw_string {
            if *i == self.alphabet.len() - 1 {
                *i = 0;
            } else {
                *i += 1;
                add_digit = false;
                break;
            }
        }

        if add_digit {
            self.raw_string.push(0);
            println!("bruteforce string length: {}", self.raw_string.len());
        }

        let string = String::from_iter(self.raw_string.iter().map(|i| self.alphabet[*i]));

        Some(string)
    }
}
