use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};

use bpers::{self, Vocabulary};

const DEFAULT_N_MERGES: u32 = 2000;

/// BPE - byte pair encoding
#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    cmd: CliCommand,
}

#[derive(Debug, Subcommand)]
enum CliCommand {
    /// Learn a vocabulary from a corpus
    Learn {
        /// Either a string or a path to an existing text file
        #[arg(value_parser = PathyString::parse)]
        input: PathyString,
        /// Max number of merges to perform during vocabulary learning
        #[arg(short = 'm', long = "merges", default_value_t = DEFAULT_N_MERGES)]
        n_merges: u32,
    },
    /// Perform text encoding
    Encode {
        /// Either a string or a path to an existing text file
        #[arg(value_parser = PathyString::parse)]
        input: PathyString,
        /// Output file for encoded text
        out: PathBuf,
        /// Max number of merges to perform during vocabulary learning. Used when no vocabulary is provided
        #[arg(short = 'm', long = "merges", default_value_t = DEFAULT_N_MERGES)]
        n_merges: u32,
        /// A path to a vocabulary binary file
        #[arg(short = 'v', long = "vocabulary", default_value = None)]
        vocabulary_path: Option<PathBuf>,
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
        CliCommand::Learn { input, n_merges } => {
            let mut vocab = Vocabulary::new();
            let input = match input {
                PathyString::Path(path) => std::fs::read_to_string(path).unwrap(),
                PathyString::String(str) => str,
            };

            _ = vocab.learn(&input, n_merges);
            println!("Learned vocabulary size: {}", vocab.id_to_token.len());
            println!("Amount of merged tokens: {}", vocab.token_pair_to_id.len());

            save_vocab(&vocab);
        }
        CliCommand::Encode {
            input,
            out,
            n_merges,
            vocabulary_path,
        } => {
            let mut vocab = Vocabulary::new();
            let input = match input {
                PathyString::Path(path) => std::fs::read_to_string(path).unwrap(),
                PathyString::String(str) => str,
            };

            let encoded = match vocabulary_path {
                Some(path) => {
                    let vocab = load_vocab(&path);
                    bpers::encode(&input, &vocab).unwrap()
                }
                None => vocab.learn(&input, n_merges),
            };

            println!("Input size:   {}", input.len());
            println!("Encoded size: {}", encoded.len());

            save_as_txt(&encoded, &out);
        }
    };
}

fn save_vocab(vocab: &Vocabulary) {
    let mut file = File::create("vocab.bin").unwrap();
    _ = bincode::encode_into_std_write(vocab, &mut file, bincode::config::standard()).unwrap();
}

fn load_vocab(from: &Path) -> Vocabulary {
    let mut file = File::open(from).unwrap();
    bincode::decode_from_std_read(&mut file, bincode::config::standard()).unwrap()
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
