/// Vale is syntax-aware linter for prose built with speed and extensibility in
/// mind.
///
/// It's designed to work many text-based formats (Markdown, reStructuredText,
/// AsciiDoc, etc.) and can be extended with custom rules and configuration.
///
/// Its functionality is exposed through a command-line interface (CLI)
/// written in Go.
///
/// This library provides high-level interface for managing Vale and its assets
/// (binary, `StylesPath`, etc.) with the goal of making it easy to add
/// IDE-like features to any text editor that supports the Language Server
/// Protocol (LSP).
pub mod error;
pub mod ini;
pub mod server;
pub mod utils;
pub mod vale;
pub mod yml;
