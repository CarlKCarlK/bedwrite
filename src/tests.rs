// https://stackoverflow.com/questions/32900809/how-to-suppress-function-is-never-used-warning-for-a-function-used-by-tests

mod tests {
    #[cfg(test)]
    use std::path::PathBuf;

    #[cfg(test)]
    use temp_testdir::TempDir;

    #[cfg(test)]
    use crate::write;

    #[test]
    fn test1() {
        let temp = TempDir::default();
        let path = PathBuf::from(temp.as_ref()).join("rust_bed_reader_writer_test.bed");
        let filename = path.as_os_str().to_str().unwrap();

        write(filename, 100, 100).unwrap();
    }
}
