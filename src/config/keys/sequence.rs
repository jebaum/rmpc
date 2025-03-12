use std::ops::RangeInclusive;

use anyhow::{Result, ensure};
use crossterm::event::{KeyCode, KeyModifiers};
use itertools::Itertools;

use super::Key;

pub struct Sequence {
    key: Key,
    followers: Vec<Sequence>,
}

#[derive(Default)]
struct Parser<'a> {
    idx: usize,
    start: usize,
    input: &'a str,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Result<Self> {
        ensure!(!input.is_empty(), "Input must not be empty");

        Ok(Self { input, idx: 0, start: 0 })
    }

    fn parse(&mut self) -> Result<Vec<Key>> {
        let chars = self.input.chars().collect_vec();
        let mut modifiers = KeyModifiers::NONE;

        let mut seq = vec![];
        loop {
            if self.idx >= chars.len() {
                break;
            }
            let c = chars[self.idx];

            let key = match c {
                '<' => {
                    let chord_range = self.chord();
                    Self::chord_to_key(&chars[chord_range])?
                }

                c => {
                    if c.is_uppercase() {
                        modifiers |= KeyModifiers::SHIFT;
                    }

                    Key { key: KeyCode::Char(c), modifiers }
                }
            };
            seq.push(key);

            self.idx += 1;
        }

        Ok(seq)
    }

    fn chord_to_key(chars: &[char]) -> Result<Key> {
        let mut idx = 0;

        // skip the surrouning '<' and '>'
        let skip_first = &chars[1..];
        let skip_last = &skip_first[..skip_first.len().saturating_sub(1)];
        let mut modifiers = KeyModifiers::NONE;

        loop {
            let Some(c) = skip_last.get(idx) else {
                break;
            };

            let next = skip_last.get(idx + 1);

            match c {
                'C' if next.is_some_and(|v| v == &'-') => {
                    modifiers |= KeyModifiers::CONTROL;
                    idx += 1;
                }
                'A' if next.is_some_and(|v| v == &'-') => {
                    modifiers |= KeyModifiers::ALT;
                    idx += 1;
                }
                'S' if next.is_some_and(|v| v == &'-') => {
                    modifiers |= KeyModifiers::SHIFT;
                    idx += 1;
                }
                _ => break,
            }
            idx += 1;
        }

        let mut skip_last = &skip_last[idx..];
        if skip_last[0] == '<' {
            let skip_first = &skip_last[1..];
            skip_last = &skip_first[..skip_first.len().saturating_sub(1)];
        }

        let key = match skip_last {
            ['B', 'S'] => KeyCode::Backspace,
            ['B', 'a', 'c', 'k', 's', 'p', 'a', 'c', 'e'] => KeyCode::Backspace,
            ['T', 'a', 'b'] if modifiers.contains(KeyModifiers::SHIFT) => KeyCode::BackTab,
            ['T', 'a', 'b'] => KeyCode::Tab,
            ['E', 'n', 't', 'e', 'r'] => KeyCode::Enter,
            ['C', 'R'] => KeyCode::Enter,
            ['R', 'e', 't', 'u', 'r', 'n'] => KeyCode::Enter,
            ['B', 's', 'l', 'a', 's', 'h'] => KeyCode::Char('\\'),
            ['B', 'a', 'r'] => KeyCode::Char('|'),

            ['l', 't'] => KeyCode::Char('<'),
            ['g', 't'] => KeyCode::Char('>'),
            ['L', 'e', 'f', 't'] => KeyCode::Left,
            ['R', 'i', 'g', 'h', 't'] => KeyCode::Right,
            ['U', 'p'] => KeyCode::Up,
            ['D', 'o', 'w', 'n'] => KeyCode::Down,
            ['H', 'o', 'm', 'e'] => KeyCode::Home,
            ['E', 'n', 'd'] => KeyCode::End,
            ['P', 'a', 'g', 'e', 'U', 'p'] => KeyCode::PageUp,
            ['P', 'a', 'g', 'e', 'D', 'o', 'w', 'n'] => KeyCode::PageDown,
            ['D', 'e', 'l'] => KeyCode::Delete,
            ['I', 'n', 's', 'e', 'r', 't'] => KeyCode::Insert,
            ['E', 's', 'c'] => KeyCode::Esc,
            ['S', 'p', 'a', 'c', 'e'] => KeyCode::Char(' '),
            ['F', '1'] => KeyCode::F(1),
            ['F', '2'] => KeyCode::F(2),
            ['F', '3'] => KeyCode::F(3),
            ['F', '4'] => KeyCode::F(4),
            ['F', '5'] => KeyCode::F(5),
            ['F', '6'] => KeyCode::F(6),
            ['F', '7'] => KeyCode::F(7),
            ['F', '8'] => KeyCode::F(8),
            ['F', '9'] => KeyCode::F(9),
            ['F', '1', '0'] => KeyCode::F(10),
            ['F', '1', '1'] => KeyCode::F(11),
            ['F', '1', '2'] => KeyCode::F(12),
            [] => KeyCode::Null,
            rest @ [c, ..] => {
                ensure!(rest.len() == 1, format!("Invalid key: '{rest:?}' from input '{chars:?}'"));

                if c.is_uppercase() {
                    modifiers |= KeyModifiers::SHIFT;
                }

                KeyCode::Char(*c)
            }
        };
        Ok(Key { key, modifiers })
    }

    fn chord(&mut self) -> RangeInclusive<usize> {
        let chars = self.input.chars().collect_vec();
        let mut open_count = 1;
        self.start = self.idx;
        self.idx += 1;

        assert!(self.idx < chars.len(), "unterminated chord");
        let mut current_char = chars[self.idx]; // possible panic

        loop {
            self.idx += 1;

            current_char = chars[self.idx];
            if current_char == '>' {
                open_count -= 1;
                if open_count == 0 {
                    break;
                }
            } else if current_char == '<' {
                open_count += 1;
            }
        }

        self.start..=self.idx
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[rustfmt::skip]
mod tests {
    use super::Parser;
    use super::*;
    use rstest::rstest;

    #[test]
    fn seq() {
        dbg!(Parser::new("<C-t>").unwrap().parse());
    }

    #[test]
    fn seq2() {
        dbg!(Parser::new("<C-t><S-w>").unwrap().parse());
    }

    #[test]
    fn seq3() {
        dbg!(Parser::new("<C-t><S-<lt>>").unwrap().parse());
    }

    #[test]
    fn seq4() {
        dbg!(Parser::new("<lt>").unwrap().parse());
    }

    #[rstest]
    //      <BS>		              backspace
    #[case("<BS>",         Key { key: KeyCode::Backspace, modifiers: KeyModifiers::NONE })]
    #[case("<C-BS>",       Key { key: KeyCode::Backspace, modifiers: KeyModifiers::CONTROL })]
    //      <Tab>		              tab
    #[case("<C-S-Tab>",    Key { key: KeyCode::BackTab,   modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT })]
    #[case("<S-Tab>",      Key { key: KeyCode::BackTab,   modifiers: KeyModifiers::SHIFT })]
    #[case("<C-Tab>",      Key { key: KeyCode::Tab,       modifiers: KeyModifiers::CONTROL })]
    #[case("<Tab>",        Key { key: KeyCode::Tab,       modifiers: KeyModifiers::NONE })]

    //      <CR>		              carriage return
    #[case("<CR>",         Key { key: KeyCode::Enter,     modifiers: KeyModifiers::NONE })]
    #[case("<C-CR>",       Key { key: KeyCode::Enter,     modifiers: KeyModifiers::CONTROL })]
    //      <Return>	              same as <CR> *<Return>*
    #[case("<Return>",     Key { key: KeyCode::Enter,     modifiers: KeyModifiers::NONE })]
    #[case("<C-Return>",   Key { key: KeyCode::Enter,     modifiers: KeyModifiers::CONTROL })]
    //      <Enter>		              same as <CR> *<Enter>*
    #[case("<Enter>",      Key { key: KeyCode::Enter,     modifiers: KeyModifiers::NONE })]
    #[case("<C-Enter>",    Key { key: KeyCode::Enter,     modifiers: KeyModifiers::CONTROL })]

    // <F1> - <F12>	function keys 1 to 12		*function_key* *function-key*
    // <S-F1> - <S-F12> shift-function keys 1 to 12	*<S-F1>*
    #[case("<C-S-F11>",    Key { key: KeyCode::F(11),     modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT })]
    #[case("<S-F11>",      Key { key: KeyCode::F(11),     modifiers: KeyModifiers::SHIFT })]
    #[case("<F11>",        Key { key: KeyCode::F(11),     modifiers: KeyModifiers::NONE })]

    //      <Up>		              cursor-up
    #[case("<Up>",         Key { key: KeyCode::Up,        modifiers: KeyModifiers::NONE })]
    //      <Down>		              cursor-down
    #[case("<Down>",       Key { key: KeyCode::Down,      modifiers: KeyModifiers::NONE })]
    //      <Left>		              cursor-left
    #[case("<Left>",       Key { key: KeyCode::Left,      modifiers: KeyModifiers::NONE })]
    //      <Right>		              cursor-right
    #[case("<Right>",      Key { key: KeyCode::Right,     modifiers: KeyModifiers::NONE })]
    //      <S-Up>		              shift-cursor-up
    #[case("<S-Up>",       Key { key: KeyCode::Up,        modifiers: KeyModifiers::SHIFT })]
    //      <S-Down>	              shift-cursor-down
    #[case("<S-Down>",     Key { key: KeyCode::Down,      modifiers: KeyModifiers::SHIFT })]
    //      <S-Left>	              shift-cursor-left
    #[case("<S-Left>",     Key { key: KeyCode::Left,      modifiers: KeyModifiers::SHIFT })]
    //      <S-Right>	              shift-cursor-right
    #[case("<S-Right>",    Key { key: KeyCode::Right,     modifiers: KeyModifiers::SHIFT })]
    //      <C-Left>	              control-cursor-left
    #[case("<C-Left>",     Key { key: KeyCode::Left,      modifiers: KeyModifiers::CONTROL })]
    //      <C-Right>	              control-cursor-right
    #[case("<C-Right>",    Key { key: KeyCode::Right,     modifiers: KeyModifiers::CONTROL })]
    #[case("<C-Up>",       Key { key: KeyCode::Up,        modifiers: KeyModifiers::CONTROL })]
    #[case("<C-Down>",     Key { key: KeyCode::Down,      modifiers: KeyModifiers::CONTROL })]

    //      <Home>		              home
    #[case("<Home>",       Key { key: KeyCode::Home,      modifiers: KeyModifiers::NONE })]
    #[case("<C-Home>",     Key { key: KeyCode::Home,      modifiers: KeyModifiers::CONTROL })]

    //      <End>		              end
    #[case("<End>",        Key { key: KeyCode::End,       modifiers: KeyModifiers::NONE })]
    #[case("<C-End>",      Key { key: KeyCode::End,       modifiers: KeyModifiers::CONTROL })]

    //      <Insert>	              insert key
    #[case("<Insert>",     Key { key: KeyCode::Insert,    modifiers: KeyModifiers::NONE })]

    //      <PageUp>	              page-up
    #[case("<PageUp>",     Key { key: KeyCode::PageUp,    modifiers: KeyModifiers::NONE })]
    #[case("<C-PageUp>",   Key { key: KeyCode::PageUp,    modifiers: KeyModifiers::CONTROL })]

    //      <PageDown>	              page-down
    #[case("<PageDown>",   Key { key: KeyCode::PageDown,  modifiers: KeyModifiers::NONE })]
    #[case("<C-PageDown>", Key { key: KeyCode::PageDown,  modifiers: KeyModifiers::CONTROL })]

    //      <lt>		              less-than
    #[case("<lt>",         Key { key: KeyCode::Char('<'), modifiers: KeyModifiers::NONE })]
    #[case("<C-S-lt>",     Key { key: KeyCode::Char('<'), modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT })]

    //      <gt>		              greater-than
    #[case("<gt>",         Key { key: KeyCode::Char('>'), modifiers: KeyModifiers::NONE })]
    #[case("<C-S-gt>",     Key { key: KeyCode::Char('>'), modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT })]

    //      <Space>		              space
    #[case("<C-Space>",    Key { key: KeyCode::Char(' '), modifiers: KeyModifiers::CONTROL })]
    #[case("<Space>",      Key { key: KeyCode::Char(' '), modifiers: KeyModifiers::NONE })]
    #[case("<C-S-Space>",  Key { key: KeyCode::Char(' '), modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT })]

    //      <Del>		              delete
    #[case("<Del>",        Key { key: KeyCode::Delete,    modifiers: KeyModifiers::NONE })]
    #[case("<C-Del>",      Key { key: KeyCode::Delete,    modifiers: KeyModifiers::CONTROL })]

    //      <Esc>		              escape
    #[case("<Esc>",        Key { key: KeyCode::Esc,       modifiers: KeyModifiers::NONE })]
    #[case("<C-Esc>",      Key { key: KeyCode::Esc,       modifiers: KeyModifiers::CONTROL })]

    //      <Bslash>	              backslash
    #[case("<Bslash>",     Key { key: KeyCode::Char('\\'), modifiers: KeyModifiers::NONE })]
    #[case("\\",           Key { key: KeyCode::Char('\\'), modifiers: KeyModifiers::NONE })]

    //      <Bar>		              vertical bar
    #[case("|",            Key { key: KeyCode::Char('|'), modifiers: KeyModifiers::NONE })]
    #[case("<Bar>",        Key { key: KeyCode::Char('|'), modifiers: KeyModifiers::NONE })]
    #[case("<S-|>",        Key { key: KeyCode::Char('|'), modifiers: KeyModifiers::SHIFT })]
    #[case("<S-Bar>",      Key { key: KeyCode::Char('|'), modifiers: KeyModifiers::SHIFT })]

    #[case("a",            Key { key: KeyCode::Char('a'), modifiers: KeyModifiers::NONE })]
    #[case("A",            Key { key: KeyCode::Char('A'), modifiers: KeyModifiers::SHIFT })]
    #[case("<C-a>",        Key { key: KeyCode::Char('a'), modifiers: KeyModifiers::CONTROL })]
    #[case("<C-A>",        Key { key: KeyCode::Char('A'), modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT })]
    #[case("5",            Key { key: KeyCode::Char('5'), modifiers: KeyModifiers::NONE })]
    #[case("<C-A-S-5>",    Key { key: KeyCode::Char('5'), modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT | KeyModifiers::ALT })]

    #[case("_",            Key { key: KeyCode::Char('_'), modifiers: KeyModifiers::NONE })]
    #[case("-",            Key { key: KeyCode::Char('-'), modifiers: KeyModifiers::NONE })]
    #[case("<C-S-->",      Key { key: KeyCode::Char('-'), modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT })]
    #[case("5",            Key { key: KeyCode::Char('5'), modifiers: KeyModifiers::NONE })]
    #[case("%",            Key { key: KeyCode::Char('%'), modifiers: KeyModifiers::NONE })]
    // #[case("",             Key { key: KeyCode::Null,      modifiers: KeyModifiers::NONE })]
    fn seq_serialization_round_trip(#[case] input: &str, #[case] expected: Key) {
        println!("input {input}");
        let deserialized = Parser::new(input).unwrap().parse().unwrap();
        // let deserialized: Key = input.parse().unwrap();
        assert_eq!(deserialized[0], expected);
    }

    #[rstest]
    #[case("<Enter>",          Key { key: KeyCode::Enter,       modifiers: KeyModifiers::NONE })]
    #[case("<Backspace>",      Key { key: KeyCode::Backspace,   modifiers: KeyModifiers::NONE })]
    fn deserialization_extras(#[case] input: &str, #[case] expected: Key) {
        let deserialized: Key = input.parse().unwrap();
        assert_eq!(deserialized, expected);
    }
}
