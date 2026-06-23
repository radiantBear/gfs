use assert_cmd::Command;
use std::{
    collections::HashSet,
    fs::File,
    io::{self, Read},
};
use tempfile::{TempDir, tempdir};

#[test]
fn it_creates_todays_file_and_maintains_gfs_archive() -> Result<(), String> {
    let setup = || -> io::Result<TempDir> {
        let dir = tempdir()?;
        File::create(dir.path().join("2025-02-14.pdf"))?;
        File::create(dir.path().join("2025-03-14.pdf"))?;
        File::create(dir.path().join("2025-04-14.pdf"))?;
        File::create(dir.path().join("2025-04-21.pdf"))?;
        File::create(dir.path().join("2025-04-28.pdf"))?;
        File::create(dir.path().join("2025-05-05.pdf"))?;
        File::create(dir.path().join("2025-05-12.pdf"))?;
        File::create(dir.path().join("2025-05-19.pdf"))?;
        File::create(dir.path().join("2025-05-20.pdf"))?;
        File::create(dir.path().join("2025-05-21.pdf"))?;
        File::create(dir.path().join("2025-05-22.pdf"))?;
        File::create(dir.path().join("2025-05-23.pdf"))?;
        Ok(dir)
    };

    let stdin_contents = "Line 1 of backup\nLine 2 with more details\n\nLine 4";
    let dir = setup().map_err(|e| e.to_string())?;

    let mut cmd = Command::cargo_bin("gfs").unwrap();
    cmd.current_dir(&dir)
        .write_stdin(stdin_contents)
        .arg("-s")
        .arg(".pdf")
        .arg("--test-fixed-date")
        .arg("2025-05-24");
    let _ = cmd.assert().success();

    let assert_files = || -> io::Result<()> {
        let mut files = HashSet::new();
        for file in dir.path().read_dir()? {
            files.insert(file?.file_name().into_string().unwrap());
        }
        let files = files;

        assert!(files.contains("2025-02-14.pdf"));
        assert!(files.contains("2025-03-14.pdf"));
        assert!(files.contains("2025-04-14.pdf"));
        assert!(files.contains("2025-04-28.pdf"));
        assert!(files.contains("2025-05-05.pdf"));
        assert!(files.contains("2025-05-12.pdf"));
        assert!(files.contains("2025-05-19.pdf"));
        assert!(files.contains("2025-05-20.pdf"));
        assert!(files.contains("2025-05-21.pdf"));
        assert!(files.contains("2025-05-22.pdf"));
        assert!(files.contains("2025-05-23.pdf"));
        assert!(files.contains("2025-05-24.pdf")); // New!

        assert!(!files.contains("2025-04-21.pdf"));

        Ok(())
    };

    let assert_contents = || -> io::Result<()> {
        let mut f = File::open(dir.path().join("2025-05-24.pdf"))?;

        let mut contents = String::new();
        f.read_to_string(&mut contents)?;

        assert_eq!(stdin_contents, contents);

        Ok(())
    };

    assert_files().map_err(|e| e.to_string())?;
    assert_contents().map_err(|e| e.to_string())?;

    dir.close().map_err(|e| e.to_string())?;
    Ok(())
}
