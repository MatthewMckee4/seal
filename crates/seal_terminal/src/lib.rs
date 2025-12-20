/// Returns the term width that insta should use.
pub fn terminal_width() -> usize {
    console::Term::stdout().size().1 as usize
}
