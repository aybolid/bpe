use std::{
    io::Write,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};

use bpers::{self, Vocabulary};

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    cmd: CliCommand,
}

#[derive(Debug, Subcommand)]
enum CliCommand {
    Encode {
        #[arg(value_parser = PathyString::parse)]
        input: PathyString,
        out: PathBuf,
        n_merges: u32,
    },
}

#[derive(Debug, Clone)]
enum PathyString {
    String(String),
    Path(PathBuf),
}

impl PathyString {
    fn parse(arg: &str) -> Result<Self, clap::Error> {
        if arg.is_empty() {
            Err(clap::Error::new(
                clap::error::ErrorKind::MissingRequiredArgument,
            ))
        } else {
            let maybe_path = PathBuf::from(arg);
            if maybe_path.exists() {
                Ok(Self::Path(maybe_path))
            } else {
                Ok(Self::String(arg.to_string()))
            }
        }
    }
}

fn main() {
    let cli = Cli::parse();
    match cli.cmd {
        CliCommand::Encode {
            input,
            out,
            n_merges,
        } => {
            let mut vocab = Vocabulary::new();

            let input = match input {
                PathyString::Path(path) => std::fs::read_to_string(path).unwrap(),
                PathyString::String(str) => str,
            };

            _ = vocab.learn(&input, n_merges);
            let encoded = bpers::encode(&input, &vocab).unwrap();
            let decoded = bpers::decode(&encoded, &vocab).unwrap();
            println!("{decoded}");

            // save_as_txt(&encoded, &out);
        }
    };
}

fn save_as_txt(data: &[u32], out_file: &Path) {
    let chars = data
        .iter()
        .map(|&c| char::from_u32(c).unwrap())
        .collect::<Vec<_>>();

    let mut file = std::fs::File::create(out_file).unwrap();
    for c in chars {
        file.write_all(c.encode_utf8(&mut [0; 4]).as_bytes())
            .unwrap();
    }
}
