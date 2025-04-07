use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Result;
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
        /// Path to an existing text file(s)
        #[arg(required = true, num_args = 1..)]
        input: Vec<PathBuf>,
        /// Output file for vocabulary
        #[arg(short = '0', long="out", default_value = DEFAULT_VOCAB_OUT)]
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
    /// Run example process to demonstrate BPE
    Example,
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

            let input = input
                .iter()
                .map(|path| match std::fs::read_to_string(path) {
                    Ok(contents) => contents,
                    Err(err) => {
                        eprintln!("Failed to load {} contents: {err}", path.display());
                        std::process::exit(1);
                    }
                })
                .collect::<Vec<String>>()
                .join(" ");

            println!("Learning");
            _ = vocab.learn(&input, n_merges);
            println!("\nLearned vocabulary size: {}", vocab.id_to_token.len());
            println!("Amount of merged tokens: {}", vocab.token_pair_to_id.len());

            if let Err(err) = save_vocab(&vocab, &out) {
                eprintln!("Failed to save vocabulary: {err}");
            };
        }
        CliCommand::Encode {
            input,
            out,
            n_merges,
            vocabulary_path,
        } => {
            let mut vocab = Vocabulary::new();
            let input = match input {
                PathyString::Path(path) => match std::fs::read_to_string(path) {
                    Ok(contents) => contents,
                    Err(err) => {
                        eprintln!("Failed to load file contents: {err}");
                        std::process::exit(1);
                    }
                },
                PathyString::String(str) => str,
            };

            let encoded = match vocabulary_path {
                Some(path) => match load_vocab(&path) {
                    Ok(vocab) => {
                        println!("Encoding");
                        match bpers::encode(&input, &vocab) {
                            Ok(encoded) => encoded,
                            Err(err) => {
                                eprintln!("Encoding failed: {err}");
                                std::process::exit(1);
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("Failed to load vocabulary: {err}");
                        std::process::exit(1);
                    }
                },
                None => {
                    println!("Learning and encoding");
                    let encoded_artifact = vocab.learn(&input, n_merges);
                    if let Err(err) = save_vocab(&vocab, &PathBuf::from(DEFAULT_VOCAB_OUT)) {
                        eprintln!("Failed to save learned vocabulary: {err}");
                    };
                    encoded_artifact
                }
            };

            println!("\nInput size:   {}", input.len());
            println!("Encoded size: {}\n", encoded.len());

            if let Err(err) = save_encoded(&encoded, &out) {
                eprintln!("Failed to save encoded data: {err}");
            };
        }
        CliCommand::Decode {
            input,
            vocabulary_path,
            out,
        } => {
            let contents = match std::fs::read_to_string(input) {
                Ok(contents) => contents,
                Err(err) => {
                    eprintln!("Failed to load file contents: {err}");
                    std::process::exit(1);
                }
            };

            let encoded = contents.chars().map(|c| c as u32).collect::<Vec<_>>();

            let vocab = match load_vocab(&vocabulary_path) {
                Ok(vocab) => vocab,
                Err(err) => {
                    eprintln!("Failed to load vocabulary: {err}");
                    std::process::exit(1);
                }
            };

            println!("Decoding\n");
            let decoded = match bpers::decode(&encoded, &vocab) {
                Ok(decoded) => decoded,
                Err(err) => {
                    eprintln!("Decoding failed: {err}");
                    std::process::exit(1);
                }
            };

            match out {
                Some(path) => {
                    if let Err(err) = save_decoded(&decoded, &path) {
                        eprintln!("Failed to save decoded data: {err}")
                    }
                }
                None => println!("{decoded}"),
            };
        }
        CliCommand::Example => {
            println!("Here is BPE in action!");
            let input = "aaabdaaabac";
            println!("Our input: {:?}\n", input);
            println!("Learning input vocabulary...");
            let mut vocab = Vocabulary::new();
            _ = vocab.learn(input, 5);
            println!("BPE performed token merges (3) and we got this vocabulary:\n");

            println!("Merge 1:  [aa]abd[aa]abac");
            println!("          |__|   |__|");
            println!("            e      e\n");

            println!("Merge 2:  e[ab]e[ab]ac");
            println!("           |__| |__|");
            println!("             f    f\n");

            println!("Merge 3:  [ef]d[ef]ac");
            println!("          |__| |__|");
            println!("            g    g\n");

            let display_vocab = vocab
                .id_to_token
                .iter()
                .map(|(id, token)| {
                    (
                        char::from_u32(*id).unwrap(),
                        match token {
                            bpers::Token::Pair(pair) => {
                                format!(
                                    "{}{}",
                                    char::from_u32(pair.left).unwrap(),
                                    char::from_u32(pair.right).unwrap()
                                )
                            }
                            bpers::Token::Lonely(lone) => {
                                char::from_u32(lone.0).unwrap().to_string()
                            }
                        },
                    )
                })
                .collect::<HashMap<_, _>>();
            println!("{:#?}", display_vocab);
            println!("Final size of vocabulary:");
            println!("\tnumber of unique characters in input + amount of performed token merges");
            println!("\t                                   4 + 3 = 7\n");
            println!("Let's encode the same input using learned vocabulary");
            println!("Encoding...");
            let encoded = bpers::encode(input, &vocab).unwrap();
            println!("Encoded input:");
            println!("\t{:?}", encoded);
            println!("\tor");
            println!(
                "\t{:?}\n",
                encoded
                    .iter()
                    .map(|id| char::from_u32(*id).unwrap())
                    .collect::<Vec<_>>()
            );
            println!(
                "Encoded size ({}) < Input size ({})\n",
                encoded.len(),
                input.len()
            );

            println!("We have vocabulary and data encoded using it.");
            println!("Lets decode encoded!");
            println!("Decoding...");
            let decoded = bpers::decode(&encoded, &vocab).unwrap();
            println!("Done! Here is the result:");
            println!(
                "\t{} -> {decoded}\n",
                encoded
                    .iter()
                    .map(|id| char::from_u32(*id).unwrap())
                    .collect::<String>()
            );
            println!("WOW! We got our input back!")
        }
    };
}

fn save_vocab(vocab: &Vocabulary, to: &Path) -> Result<()> {
    println!("Saving vocabulary to {}", to.display());
    let mut file = File::create(to)?;
    _ = bincode::encode_into_std_write(vocab, &mut file, bincode::config::standard())?;
    Ok(())
}

fn load_vocab(from: &Path) -> Result<Vocabulary> {
    println!("Loading vocabulary from {}", from.display());
    let mut file = File::open(from)?;
    let vocab = bincode::decode_from_std_read(&mut file, bincode::config::standard())?;
    Ok(vocab)
}

fn save_encoded(data: &[u32], to: &Path) -> Result<()> {
    println!("Saving encoded data to {}", to.display());
    let chars = data
        .iter()
        .map(|&c| char::from_u32(c).unwrap())
        .collect::<Vec<_>>();

    let mut file = std::fs::File::create(to)?;
    for c in chars {
        file.write_all(c.encode_utf8(&mut [0; 4]).as_bytes())?;
    }
    Ok(())
}

fn save_decoded(data: &str, to: &Path) -> Result<()> {
    println!("Saving decoded data to {}", to.display());
    let mut file = std::fs::File::create(to)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}
