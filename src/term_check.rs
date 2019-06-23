use std::io::{stdin, stdout, Error};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::Style;
use tui::widgets::{Block, Borders, Paragraph, Text, Widget};
use tui::Terminal;

pub fn resolution_check() -> Result<(), Error> {
    let stdout = stdout().into_raw_mode()?;
    let screen = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(screen);
    let mut terminal = Terminal::new(backend)?;
    let mut need_input = false;

    loop {
        terminal.draw(|mut f| {
            let size = f.size();
            let height = size.height;
            let width = size.width;

            let recommended_height = 30;
            let recommended_width = 80;

            if height < recommended_height || width < recommended_width {
                need_input = true;
            }

            if need_input {
                let root_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(100)].as_ref())
                    .split(f.size());
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(5)
                    .constraints([Constraint::Percentage(100)].as_ref())
                    .split(root_layout[0]);
                let passage_block = Block::default()
                    .borders(Borders::ALL)
                    .title_style(Style::default());
                Paragraph::new(
                    [Text::raw(format!(
                        "Terminal width and height too small!\nwidth: {}\nheight: {}\n\nIt is strongly recommended to play this with a height of at least: {} and a width of at least: {}\nConsider making your terminal fullscreen!\n\nCheck again <ENTER>, Ignore check: ^D, Exit: ^C",
                        width,
                        height,
                        recommended_height,
                        recommended_width
                    ))]
                    .iter(),
                )
                .block(passage_block.clone().title("Checking bounds"))
                .wrap(true)
                .alignment(Alignment::Left)
                .render(&mut f, chunks[0]);
            }
        })?;

        if need_input {
            need_input = false;
            let stdin = stdin();
            for c in stdin.keys() {
                let checked = c.unwrap();
                if checked == Key::Ctrl('c') {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "User wants to exit",
                    ));
                }
                if checked == Key::Ctrl('d') {
                    return Ok(());
                }
                if checked == Key::Char('\n') {
                    break;
                }
            }
        } else {
            return Ok(());
        }
    }
}
