/// Returns the term width that insta should use.
pub fn terminal_width() -> usize {
    use std::io::IsTerminal;

    if std::io::stdout().is_terminal() {
        console::Term::stdout().size().1 as usize
    } else {
        80 // Default width for non-interactive (like tests, pipes, etc.)
    }
}
