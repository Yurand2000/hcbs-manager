pub struct TempDir {
    path: std::path::PathBuf
}

impl TempDir {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<Self> {
        std::fs::create_dir(path.as_ref())
            .map(|_| Self { path: path.as_ref().to_owned() })
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        std::fs::remove_dir(self.path.as_path()).unwrap()
    }
}