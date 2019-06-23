pub struct Argument {
    /// how many times a word repeat itself
    pub repeat: u8,
    /// wpm speed threshold, if lower than this, show a message
    pub speed_threshold: u8,
    /// the `.anki2` anki deck file path
    pub anki_deck: String,    
    /// the `input` for the game
    pub input: String,
    /// if present, only type the word, no explanation
    pub word_only: bool,
    /// a history for typed word
    pub typed_words: Vec<String>,
    /// the current word user is typing
    pub current_word: String,
    /// specified words.
    pub words: String,
    /// sequentially show the words
    pub sequential: bool,
    /// starting word if sequential
    pub starting_word: String,

    /// current id which is only available in sequential mode
    pub current_id: i64,
}