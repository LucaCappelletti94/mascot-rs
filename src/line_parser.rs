pub trait LineParser {

    /// Returns `true` if the line can be parsed by the data structure.
    fn can_parse_line(&self, line: &str) -> bool;

    /// Parses the line and updates the data structure.
    fn digest_line(&mut self, line: &str) -> Result<(), String>;
}
