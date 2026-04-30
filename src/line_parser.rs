/// A line-oriented parser used by the MGF builders.
pub trait LineParser {
    /// Returns `true` if the line can be parsed by the data structure.
    fn can_parse_line(line: &str) -> bool;

    /// Parses the line and updates the data structure.
    ///
    /// # Errors
    /// Returns an error if the line cannot be parsed or violates the parser's
    /// current state.
    fn digest_line(&mut self, line: &str) -> Result<(), String>;

    /// Returns whether the data structure can be built.
    fn can_build(&self) -> bool;
}
