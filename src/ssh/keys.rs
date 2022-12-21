#[derive(Debug, Clone)]
pub enum Key {
    Enter,
    Tab,
    BackTab, // yes i feel backstabbed too, you are not alone =)
    Delete,
    Up,
    Left,
    Right,
    Down,
    SelectUp,
    SelectLeft,
    SelectDown,
    SelectRight,
    TotalLeft,
    TotalRight,
    NextUp,
    NextLeft,
    NextRight,
    NextDown,
    Interrupt,
    Eof,
    Char(char),
    MultiChar(String),
}

impl From<&str> for Key {
    fn from(key: &str) -> Self {
        match key {
            "\r" => Key::Enter,
            "\t" => Key::Tab,
            "\u{1b}[Z" => Key::BackTab,
            "\u{7f}" => Key::Delete,
            "\u{1b}[A" => Key::Up,
            "\u{1b}[B" => Key::Left,
            "\u{1b}[C" => Key::Right,
            "\u{1b}[D" => Key::Down,
            "\u{1b}[1;2A" => Key::SelectUp,
            "\u{1b}[1;2B" => Key::SelectLeft,
            "\u{1b}[1;2C" => Key::SelectRight,
            "\u{1b}[1;2D" => Key::SelectDown,
            "\u{1}" => Key::TotalLeft,
            "\u{5}" => Key::TotalRight,
            "\u{1b}[1;3A" => Key::NextUp,
            "\u{1b}[1;3B" => Key::NextDown,
            "\u{1b}b" => Key::NextLeft,
            "\u{1b}f" => Key::NextRight,
            "\u{3}" => Key::Interrupt,
            "\u{4}" => Key::Eof,
            _ => {
                if key.len() == 1 {
                    Key::Char(key.chars().next().unwrap())
                } else {
                    Key::MultiChar(key.to_owned())
                }
            }
        }
    }
}
