use std::path::Path;
use std::fs::File;
use std::io;

struct ConfigReader {}

impl ConfigReader {
    pub fn read(path: Path) -> io::Result<Self> {
        let file = File::open(path)?;
        let metadata = file.metadata()?;

    }
}