use clap;
use std::io::Error;
use std::path::Path;
use std::env::current_dir;

mod game;
mod term_check;
mod option;

fn main() -> Result<(), Error> {
    let path = format!("{}/asserts/collection.anki2", current_dir()?.display());

    let args = clap::App::new("Terminal Typing English, memorize English vocabulary by typing.")
        .version("0.1")
        .author("Leo Liu <leo.wei.liu@gmail.com>")
        .setting(clap::AppSettings::TrailingVarArg)
        .arg(
            clap::Arg::with_name("REPEAT")
            .short("r")
            .long("repeat")
            .multiple(false)
            .required(false)
            .takes_value(true)
            .default_value("5")
            .help("How many times a English word repeats")
        )
        .arg(
            clap::Arg::with_name("SPEED")
            .long("speed")
            .multiple(false)
            .required(false)
            .takes_value(true)
            .default_value("20")
            .help("If your typing speed is lower than this value, you have to restart typing this word.")
        )
        .arg(
            clap::Arg::with_name("ANKI_DECK")
            .short("d")
            .long("deck-path")
            .multiple(false)
            .required(false)
            .takes_value(true)
            .default_value(path.as_str())
            .help("Specify the anki deck file path.")
        )
        .arg(
            clap::Arg::with_name("WORDS")
            .short("w")
            .long("words")
            .multiple(false)
            .required(false)
            .takes_value(true)
            .default_value("")
            .help("Specify words you want to type, separate by comma")
        )
        .arg(
            clap::Arg::with_name("WORD_ONLY")
            .long("word-only")
            .multiple(false)
            .required(false)
            .takes_value(false)
            .help("Only type the word, no explanation.")
        )
        .arg(
            clap::Arg::with_name("SEQUENTIAL")
            .long("sequential")
            .multiple(false)
            .required(false)
            .takes_value(false)
            .help("Sequentially show the words, not randomly")
        )
        .arg(
            clap::Arg::with_name("FROM")
            .long("from")
            .multiple(false)
            .required(false)
            .takes_value(true)
            .default_value("a")
            .help("In sequential mode, which word to start with")
        )
        .get_matches();
    let anki_deck_path = args.value_of("ANKI_DECK").unwrap();

    if !Path::new(args.value_of("ANKI_DECK").unwrap()).exists() {
        panic!("The Anki deck file: {} does not exist.", anki_deck_path);
    }

    let mut option = option::Argument {
        repeat: args.value_of("REPEAT").unwrap().parse::<u8>().unwrap(),
        speed_threshold: args.value_of("SPEED").unwrap().parse::<u8>().unwrap(),
        anki_deck: anki_deck_path.to_owned(),
        input: "".to_owned(),
        word_only: args.is_present("WORD_ONLY"),
        typed_words: vec![],
        current_word: "".to_owned(),
        words: args.value_of("WORDS").unwrap().to_owned(),
        sequential: args.is_present("SEQUENTIAL"),
        starting_word: args.value_of("FROM").unwrap().to_owned(),
        current_id: 0
    };

    if term_check::resolution_check().is_ok() {
        while game::play_game(&mut option) {
            // Ensure the text passed is only played the first time
            option.input = "".to_owned();
        }
    }
    Ok(())
}
