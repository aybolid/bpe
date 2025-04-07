use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};

use bpers::{self, Vocabulary};

const DEFAULT_N_MERGES: u32 = 2000;
const DEFAULT_VOCAB_OUT: &str = "vocab.bin";
const DEFAULT_ENCODED_OUT: &str = "encoded.txt";

/// BPE - byte pair encoding
#[derive(Debug, Parser)]
#[command(version)]
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
        /// Output file for vocabulary
        #[arg(default_value = DEFAULT_VOCAB_OUT)]
        out: PathBuf,
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
        #[arg(default_value = DEFAULT_ENCODED_OUT)]
        out: PathBuf,
        /// A path to a vocabulary binary file
        #[arg(short = 'v', long = "vocabulary", default_value = None)]
        vocabulary_path: Option<PathBuf>,
        /// Max number of merges to perform during vocabulary learning. Used when no vocabulary is provided
        #[arg(short = 'm', long = "merges", default_value_t = DEFAULT_N_MERGES)]
        n_merges: u32,
    },
    /// Decode using provided vocabulary
    Decode {
        /// Input file with encoded text
        input: PathBuf,
        /// A path to a vocabulary binary file
        #[arg(short = 'v', long = "vocabulary")]
        vocabulary_path: PathBuf,
        /// Out for decoded text. Stdout if not provided
        #[arg(short = 'o', long = "out", default_value = None)]
        out: Option<PathBuf>,
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
        CliCommand::Learn {
            input,
            out,
            n_merges,
        } => {
            let mut vocab = Vocabulary::new();
            let input = match input {
                PathyString::Path(path) => std::fs::read_to_string(path).unwrap(),
                PathyString::String(str) => str,
            };

            println!("Learning");
            _ = vocab.learn(&input, n_merges);
            println!("\nLearned vocabulary size: {}", vocab.id_to_token.len());
            println!("Amount of merged tokens: {}", vocab.token_pair_to_id.len());

            save_vocab(&vocab, &out);
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
                    println!("Encoding");
                    bpers::encode(&input, &vocab).unwrap()
                }
                None => {
                    println!("Learning and encoding");
                    let encoded_artifact = vocab.learn(&input, n_merges);
                    save_vocab(&vocab, &PathBuf::from(DEFAULT_VOCAB_OUT));
                    encoded_artifact
                }
            };

            println!("\nInput size:   {}", input.len());
            println!("Encoded size: {}\n", encoded.len());

            save_encoded(&encoded, &out);
        }
        CliCommand::Decode {
            input,
            vocabulary_path,
            out,
        } => {
            let input = std::fs::read_to_string(&input)
                .unwrap()
                .chars()
                .map(|c| c as u32)
                .collect::<Vec<_>>();
            let vocab = load_vocab(&vocabulary_path);

            println!("Decoding\n");
            let decoded = bpers::decode(&input, &vocab).unwrap();

            match out {
                Some(path) => save_decoded(&decoded, &path),
                None => println!("{decoded}"),
            }
        }
    };
}

fn save_vocab(vocab: &Vocabulary, to: &Path) {
    println!("Saving vocabulary to {}", to.display());
    let mut file = File::create(to).unwrap();
    _ = bincode::encode_into_std_write(vocab, &mut file, bincode::config::standard()).unwrap();
}

fn load_vocab(from: &Path) -> Vocabulary {
    println!("Loading vocabulary from {}", from.display());
    let mut file = File::open(from).unwrap();
    bincode::decode_from_std_read(&mut file, bincode::config::standard()).unwrap()
}

fn save_encoded(data: &[u32], to: &Path) {
    println!("Saving encoded data to {}", to.display());
    let chars = data
        .iter()
        .map(|&c| char::from_u32(c).unwrap())
        .collect::<Vec<_>>();

    let mut file = std::fs::File::create(to).unwrap();
    for c in chars {
        file.write_all(c.encode_utf8(&mut [0; 4]).as_bytes())
            .unwrap();
    }
}

fn save_decoded(data: &str, to: &Path) {
    println!("Saving decoded data to {}", to.display());
    let mut file = std::fs::File::create(to).unwrap();
    file.write_all(data.as_bytes()).unwrap();
}
