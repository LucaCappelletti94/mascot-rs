pub trait LineParser {

    /// Returns `true` if the line can be parsed by the data structure.
    fn can_parse_line(line: &str) -> bool;

    /// Parses the line and updates the data structure.
    fn digest_line(&mut self, line: &str) -> Result<(), String>;

    /// Returns whether the data structure can be built.
    fn can_build(&self) -> bool;
}
