use std::{
    fs::{OpenOptions, read_dir, remove_file},
    io::{self, Read},
    path::Path,
};

/// Gets all files in the current directory and invokes `parse` to parse their filenames.
/// Any files whose names fail to parse are excluded from the resulting vector of parsed
/// results.
pub(crate) fn parse_files<F, R>(path: impl AsRef<Path>, mut parse: F) -> io::Result<Vec<R>>
where
    F: FnMut(&str) -> Result<R, String>,
{
    let mut dates = Vec::new();

    let files = read_dir(path)?;

    for entry in files {
        let path = entry?.path();
        if path.is_file() {
            let Some(file) = path.file_name() else {
                eprintln!("No filename found in {:?}", path);
                continue;
            };
            let Some(file) = file.to_str() else {
                eprintln!("Invalid Unicode in \"{:#?}\"", path);
                continue;
            };
            let Ok(date) = parse(file) else {
                eprintln!("Unable to parse \"{:#?}\"", path);
                continue;
            };

            dates.push(date);
        }
    }

    Ok(dates)
}

/// Copies all data from the input stream (e.g. `stdin`) into the given file, overwriting
/// the file's current contents
pub(crate) fn copy_input_to_file(input: &mut impl Read, path: impl AsRef<Path>) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;

    io::copy(input, &mut file)?;

    Ok(())
}

/// Removes all files whose name is in `names`
pub(crate) fn remove_files<P: AsRef<Path>>(path: P, names: Vec<String>) -> io::Result<()> {
    for name in names {
        remove_file(path.as_ref().join(name))?
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashSet,
        fs::File,
        io::{Cursor, Read as _, Seek as _, SeekFrom},
    };

    use tempfile::{NamedTempFile, tempdir};

    use super::*;

    #[test]
    fn it_parsing_files_using_provided_callback() -> io::Result<()> {
        let mut calls = HashSet::new();
        let parse = |s: &str| {
            calls.insert(s.to_owned());
            Ok(())
        };

        let dir = tempdir()?;
        File::create(dir.path().join("2025-02-28"))?;
        File::create(dir.path().join("2025-05-01"))?;
        File::create(dir.path().join("2025-05-02"))?;
        File::create(dir.path().join("2025-05-03"))?;
        File::create(dir.path().join("2025-05-31"))?;
        File::create(dir.path().join("2025-06-01"))?;

        parse_files(&dir, parse)?;

        assert!(calls.contains("2025-02-28"));
        assert!(calls.contains("2025-05-01"));
        assert!(calls.contains("2025-05-02"));
        assert!(calls.contains("2025-05-03"));
        assert!(calls.contains("2025-05-31"));
        assert!(calls.contains("2025-06-01"));

        dir.close()?;
        Ok(())
    }

    #[test]
    fn it_stores_input_in_given_file() -> io::Result<()> {
        let mock_stdin_contents = "line 1\nsome more stuff\nfinal line";
        let mut mock_stdin = Cursor::new(mock_stdin_contents);
        let mut path = NamedTempFile::new()?;

        copy_input_to_file(&mut mock_stdin, &path)?;

        path.seek(SeekFrom::Start(0))?;
        let mut result = String::new();
        path.read_to_string(&mut result)?;

        assert_eq!(mock_stdin_contents, result);

        Ok(())
    }

    #[test]
    fn it_removes_specified_files() -> io::Result<()> {
        let dir = tempdir()?;
        File::create(dir.path().join("a"))?;
        File::create(dir.path().join("b"))?;
        File::create(dir.path().join("c"))?;
        File::create(dir.path().join("d"))?;
        File::create(dir.path().join("e"))?;
        File::create(dir.path().join("f"))?;

        remove_files(&dir, vec!["b".to_owned(), "d".to_owned(), "f".to_owned()])?;

        let mut files = HashSet::new();
        for file in dir.path().read_dir()? {
            files.insert(file?.file_name().into_string().unwrap());
        }
        let files = files;

        assert!(files.contains("a"));
        assert!(files.contains("c"));
        assert!(files.contains("e"));
        assert!(!files.contains("b"));
        assert!(!files.contains("d"));
        assert!(!files.contains("f"));

        dir.close()?;
        Ok(())
    }
}
