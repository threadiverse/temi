//! Library utilities.

use crate::Result;

/// Convenience function for writing bytes to a file.
///
/// Mostly helpful for debugging API endpoint responses.
pub fn write_to_file(file_name: &str, bytes: &[u8]) -> Result<()> {
    use std::io::Write;

    let mut file = std::fs::File::create(file_name)?;
    file.write_all(bytes)?;

    Ok(())
}
