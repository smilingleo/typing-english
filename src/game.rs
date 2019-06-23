#![allow(clippy::all)]

use rand::Rng;
use std::io::{stdin, stdout, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use termion::cursor::{Left, Right};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::Color;
use tui::style::{Modifier, Style};
use tui::widgets::{Block, Borders, Paragraph, Text, Widget};
use tui::Terminal;
use sqlite::{Connection};
use std::path::Path;
use regex::Regex;
// use `crate` to use mod of this project.
use crate::option::Argument;

use dissolve::strip_html_tags;

fn get_wpm(word_count: usize, duration: u64, start_time: u64) -> u64 {
    let minute_float = ((duration - start_time) as f64) / 60.0;
    let word_count_float = (word_count + 1) as f64;
    (word_count_float / minute_float) as u64
}

fn check_word(word: &str, input: &String) -> bool {
    return *word == *input;
}

#[allow(unused_assignments)]
fn get_passage(option: &mut Argument) -> (String, String, String) {
    let path = Path::new(option.anki_deck.as_str());
    let connection = Connection::open(path).unwrap();
    let mut query = "".to_owned();

    // if user just wants to test specific words.
    if option.words.len() > 0 {
        // in case user specify words, iterate the words, currend_word as a cursor
        let words: Vec<&str> = option.words.as_str().split(",").map(|w|w.trim()).collect();
        let index = words.iter().position(|w|*w == option.current_word.as_str()).unwrap_or(0);
        if index == 0 && option.current_word.len() == 0 {
            option.current_word = words.get(index).unwrap_or(&"a").to_owned().to_owned();
        } else {
            option.current_word = words.get(index + 1).unwrap_or(&"a").to_owned().to_owned();
        }

        // words are finished, start it over
        if !words.contains(&option.current_word.as_str()) {
            option.current_word = words.get(0).unwrap_or(&"a").to_owned().to_owned();
        }
        query = format!("where sfld='{}'", option.current_word);
    } else {
        if option.sequential {
            // for sequential mode, query the first id first
            if option.current_id == 0 {
                let _ = connection.iterate(format!("select id from notes where sfld='{}'", option.starting_word), |row| {
                    for &(_, value) in row.iter() {
                        let id_value = value.unwrap().parse::<i64>().unwrap_or(-1);
                        if id_value < 0 {
                            panic!("can not find word: {}", option.starting_word);
                        }
                        option.current_id = id_value - 1;
                    }
                    true
                });
            }
            query = format!("where id={}", option.current_id + 1);
            option.current_id += 1; // move forward
        } else {
            let mut total = 0;
            let _ = connection.iterate("select count(*) as cnt from notes", |row| {
                let cnt_str = row[0].1.unwrap_or("0");
                // convert &str to usize
                total = cnt_str.parse::<i32>().unwrap();
                true
            });

            if total <= 0 {
                panic!("there might be no deck in this file, total deck: {}", total);
            }
            let rnd = rand::thread_rng().gen_range(0, total - 11);
            query = format!("limit {},1", rnd);
        }
    }


    //FIXME: This is COA specific pattern.
    let re = Regex::new(r"([a-z]+).*<div style='color:BlueViolet'>(.+);</div>.*<div style='color:DeepSkyBlue'>(.+)\.</div>.*").unwrap();
    let mut rtn: (String, String, String) = ("".to_owned(), "".to_owned(), "".to_owned());
    let _ = connection.iterate(format!("select flds from notes {}", query), |row| {
        for &(_, value) in row.iter() {
            let content = value.unwrap();
            let no_tag_content = strip_html_tags(content).join("\n");
            for group in re.captures_iter(content) {
                let word = group.get(1).unwrap().as_str();
                option.current_word = word.to_owned();

                let words = format!("{} ", word);
                let content = if option.word_only { 
                    words.as_str().repeat(option.repeat as usize) 
                } else { 
                    format!("{}. {}.", words.as_str().repeat(option.repeat as usize), group.get(3).unwrap().as_str()) 
                };
                rtn = (content.as_str().to_owned(), group.get(2).unwrap().as_str().to_owned(), no_tag_content.to_owned());
            }
        }
        true
    });

    return rtn;
}

// Get formatted version of a single word in a passage and the user's current input
// All similar characters up until the first different character are highlighted with green
// The first error character in the word is highlighted with red and the rest unformatted.
// The entire error is colored red on the user's input.
// returns a tuple with the formatted version of the: word and the input
fn get_formatted_words(word: &str, input: &str) -> (Vec<Text<'static>>, Vec<Text<'static>>) {
    let indexable_word: Vec<char> = word.chars().collect();
    let indexable_input: Vec<char> = input.chars().collect();
    let idx_word_count = indexable_word.len();
    let idx_input_count = indexable_input.len();

    let mut formatted_word: Vec<Text> = Vec::new();
    let mut formatted_input: Vec<Text> = Vec::new();
    let mut word_dex = 0;

    while word_dex < idx_word_count && word_dex < idx_input_count {
        if indexable_word[word_dex] != indexable_input[word_dex] {
            break;
        }

        formatted_word.push(Text::styled(
            indexable_word[word_dex].to_string(),
            Style::default().fg(Color::Green),
        ));
        formatted_input.push(Text::styled(
            indexable_word[word_dex].to_string(),
            Style::default().fg(Color::Green),
        ));

        word_dex += 1;
    }

    // Fill out whatever is left (the user has made a mistake for the rest of the word)

    // Only show the first error the user made in the passage (if there is any)
    let mut err_first_char = idx_input_count >= idx_word_count;
    for i in word_dex..idx_word_count {
        if err_first_char {
            formatted_word.push(Text::styled(
                indexable_word[i].to_string(),
                Style::default().bg(Color::Red).fg(Color::White),
            ));
            err_first_char = false;
        } else {
            formatted_word.push(Text::raw(indexable_word[i].to_string()));
        }
    }

    // Make all of the user's typed error red
    for i in word_dex..idx_input_count {
        formatted_input.push(Text::styled(
            indexable_input[i].to_string(),
            Style::default().bg(Color::Red).fg(Color::White),
        ));
    }

    (formatted_word, formatted_input)
}

// Gets index of fully formatted text where the word the user is typing starts.
fn get_starting_idx(words: &Vec<&str>, current_word_idx: &usize) -> usize {
    let mut passage_starting_idx: usize = 0;
    for i in 0..*current_word_idx {
        passage_starting_idx += words[i].chars().count() + 1
    }
    passage_starting_idx
}

// Get fully formatted versions of the passage, and the user's input.
fn get_formatted_texts(
    words: &Vec<&str>,
    user_input: &String,
    current_word_idx: &usize,
    mut formatted_text: Vec<Text<'static>>,
) -> (Vec<Text<'static>>, Vec<Text<'static>>) {
    let (formatted_passage_word, formatted_user_input) =
        get_formatted_words(words[*current_word_idx].clone(), user_input);

    let starting_idx = get_starting_idx(words, current_word_idx);

    for i in 0..formatted_passage_word.len() {
        formatted_text[starting_idx + i] = formatted_passage_word[i].clone();
    }
    (formatted_text, formatted_user_input)
}
//TODO: if wpm is lower than threshold, do it again.
fn get_complete_string(wpm: u64, option: &mut Argument) -> Vec<Text<'static>> {
    option.typed_words.insert(0, format!("  {}\n", option.current_word));

    let hint = Text::raw("^N to play again, ^C to quit");
    if wpm < option.speed_threshold as u64{
        vec![
            Text::styled(
                format!("You are too slow! {} < {}\n", wpm, option.speed_threshold),
                Style::default().bg(Color::Green).fg(Color::White),
            ),
            hint,
        ]        
    } else {
        vec![
            Text::styled(
                "COMPLETE\n",
                Style::default().bg(Color::Green).fg(Color::White),
            ),
            hint,
        ]
    }

}

pub fn play_game(option: &mut Argument) -> bool {
    let input = option.input.as_str();
    let stdout = stdout()
        .into_raw_mode()
        .expect("Failed to manipulate terminal to raw mode");
    let screen = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(screen);
    let mut terminal = Terminal::new(backend).expect("Failed to get handle to terminal");

    let (raw_passage, raw_title, content) = match input {
        "" => get_passage(option),
        _ => (input.to_string(), "Terminal Typeracer".to_string(), "Failed to get content from Anki Deck.".to_owned()),
    };

    let mut formatted_passage: Vec<Text> = raw_passage
        .chars()
        .map(|it| Text::raw(it.to_string()))
        .collect();

    let mut user_input = String::new();
    let mut formatted_user_input: Vec<Text> = vec![];

    // Split the passager into vec of words to work on one at a time
    let words: Vec<&str> = raw_passage.split(' ').collect();
    let mut current_word_idx = 0;

    // Timing and wpm
    let mut wpm = 0;
    let mut start_time = 0;

    loop {
        let stdin = stdin();
        terminal
            .draw(|mut f| {
                let root_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(5), Constraint::Percentage(45), Constraint::Percentage(50)].as_ref())
                    .split(f.size());
                {
                    // Title
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        // use empty column as margin
                        .constraints([Constraint::Percentage(5), Constraint::Percentage(90), Constraint::Percentage(5)].as_ref())
                        .split(root_layout[0]);

                    let passage_block = Block::default()
                        .borders(Borders::NONE)
                        .title_style(Style::default());
                    Paragraph::new([Text::raw("Typing English")].iter())
                        .block(passage_block.clone().title(""))
                        .wrap(true)
                        .alignment(Alignment::Center)
                        .render(&mut f, chunks[1]);
                }
                {
                    // Full content
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        // use empty column as margin
                        .constraints([Constraint::Percentage(5), Constraint::Percentage(90), Constraint::Percentage(5)].as_ref())
                        .split(root_layout[1]);

                    let formatted_content: Vec<Text> = content.chars().map(|it| Text::raw(it.to_string())).collect();
                    let passage_block = Block::default()
                        .borders(Borders::ALL)
                        .title_style(Style::default());
                    Paragraph::new(formatted_content.iter())
                        .block(passage_block.clone().title(""))
                        .wrap(true)
                        .alignment(Alignment::Left)
                        .render(&mut f, chunks[1]);
                }
                {
                    // Typing area.
                    let root_layout = Layout::default()
                        .direction(Direction::Horizontal)
                        .margin(0)
                        .constraints([Constraint::Percentage(5), Constraint::Percentage(70), Constraint::Percentage(20), Constraint::Percentage(5)].as_ref())
                        .split(root_layout[2]);                    
                    {
                        // Typing layout (column 1)
                        let chunks = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                            .split(root_layout[1]);
                        let passage_block = Block::default()
                            .borders(Borders::ALL)
                            .title_style(Style::default());
                        Paragraph::new(formatted_passage.iter())
                            .block(passage_block.clone().title(&raw_title))
                            .wrap(true)
                            .alignment(Alignment::Left)
                            .render(&mut f, chunks[0]);

                        let typing_block = Block::default()
                            .borders(Borders::ALL)
                            .title_style(Style::default().modifier(Modifier::BOLD));
                        Paragraph::new(formatted_user_input.iter())
                            .block(typing_block.clone().title("Type out passage here"))
                            .wrap(true)
                            .alignment(Alignment::Left)
                            .render(&mut f, chunks[1]);
                    }
                    {
                        // right panel. (column 2)
                        let chunks = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([Constraint::Percentage(50), Constraint::Percentage(20), Constraint::Percentage(30)].as_ref())
                            .split(root_layout[2]);

                        let word_list_block = Block::default()
                            .borders(Borders::ALL)
                            .title_style(Style::default());
                        let typed_words: Vec<Text> = option.typed_words.clone().into_iter().map(|w|Text::raw(w)).collect();
                        Paragraph::new(typed_words.iter())
                            .block(word_list_block.clone().title("Typed"))
                            .alignment(Alignment::Left)
                            .render(&mut f, chunks[0]);

                        let wpm_block = Block::default()
                            .borders(Borders::ALL)
                            .title_style(Style::default());
                        Paragraph::new([Text::raw(format!("WPM\n{}", wpm))].iter())
                            .block(wpm_block.clone().title("WPM"))
                            .alignment(Alignment::Center)
                            .render(&mut f, chunks[1]);

                        let help_block = Block::default()
                            .borders(Borders::ALL)
                            .title_style(Style::default());
                        Paragraph::new([Text::raw(format!("^N: Next\n^C: Exit\n\n@smilingleo"))].iter())
                            .block(help_block.clone().title("Help"))
                            .alignment(Alignment::Left)
                            .render(&mut f, chunks[2]);

                    }
                }
            })
            .expect("Failed to draw terminal widgets.");

        if current_word_idx == words.len() {
            break;
        }

        for c in stdin.keys() {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards");
            match c.unwrap() {
                Key::Ctrl('c') => return false,
                Key::Ctrl('n') => return true,
                Key::Backspace => {
                    user_input.pop();
                    if user_input.chars().count() > 0 {
                        write!(terminal.backend_mut(), "{}", Left(1))
                            .expect("Failed to write to terminal.");
                    }
                    break;
                }
                Key::Char(c) => {
                    if start_time == 0 {
                        start_time = now.as_secs() - 1;
                    }

                    if c == ' ' && check_word(words[current_word_idx], &user_input) {
                        current_word_idx += 1;
                        // BUG: Cursor stays in a forward position after clearing
                        // As soon as the user types it goes back to the beginning position
                        // Moving the cursor manually to the left does not fix
                        user_input.clear();
                    } else if c == '\n' || c == '\t' {
                        // Ignore a few types that can put the user in a weird spot
                        break;
                    } else {
                        user_input.push(c);
                        write!(terminal.backend_mut(), "{}", Right(1))
                            .expect("Failed to write to terminal.");
                    }
                    wpm = get_wpm(current_word_idx, now.as_secs(), start_time);
                    break;
                }
                _ => {
                    break;
                }
            }
        }

        if current_word_idx == words.len() {
            // We want one more render cycle at the end.
            // Ignore the dangerous function call, and then do another bounds check and break
            // before taking user input again.
            user_input.clear();
            formatted_user_input = get_complete_string(wpm, option);            
        } else if current_word_idx + 1 == words.len()
            && check_word(words[current_word_idx], &user_input)
        {
            // Special case for the last word so the user doesn't need to hit space
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards");

            current_word_idx += 1;
            wpm = get_wpm(current_word_idx, now.as_secs(), start_time);
            user_input.clear();
            formatted_user_input = get_complete_string(wpm, option);
        } else {
            let (return_passage, return_input) = get_formatted_texts(
                &words,
                &user_input.to_string(),
                &current_word_idx,
                formatted_passage,
            );

            formatted_passage = return_passage;
            formatted_user_input = return_input;
        }
    }

    loop {
        let stdin = stdin();
        for c in stdin.keys() {
            let checked = c.unwrap();
            if checked == Key::Ctrl('c') {
                return false;
            }
            if checked == Key::Ctrl('n') {
                return true;
            }
        }
    }
}
