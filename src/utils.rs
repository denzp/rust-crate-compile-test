use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use failure::{Error, ResultExt};
use walkdir::WalkDir;

pub trait SourceCodeAnalyser<T> {
    fn analyse_crate(path: &Path) -> Result<Vec<T>, Error> {
        let sources = WalkDir::new(&path.join("src"))
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                let extension = entry.path().extension().and_then(|item| item.to_str());

                match extension {
                    Some("rs") => Some(PathBuf::from(entry.path())),
                    _ => None,
                }
            });

        let mut result = Vec::new();

        for file_path in sources {
            result.append(&mut Self::analyse_file(&path, &file_path)?);
        }

        Ok(result)
    }

    fn analyse_file(crate_path: &Path, file_path: &Path) -> Result<Vec<T>, Error> {
        let source_path = file_path.strip_prefix(crate_path)?;
        let source_file = BufReader::new({
            File::open(&file_path).context(format!("Unable to open source at {:?}", file_path))?
        });

        let mut line_num = 1;

        Ok(source_file.lines().fold(vec![], |mut result, line| {
            match Self::analyse_source_line(&result, &source_path, (line_num, &line.unwrap())) {
                Ok(Some(item)) => {
                    result.push(item);
                }

                _ => {}
            }

            line_num += 1;
            result
        }))
    }

    fn analyse_source_line(
        previous: &[T],
        path: &Path,
        line: (usize, &str),
    ) -> Result<Option<T>, Error>;
}
