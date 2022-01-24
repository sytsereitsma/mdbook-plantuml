#[cfg(test)]
use std::path::{Path, PathBuf};

#[cfg(test)]
pub fn join_path<LP: AsRef<Path>, RP: AsRef<Path>>(left: LP, right: RP) -> PathBuf {
    let p = left.as_ref().to_path_buf();
    p.join(right.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_join_path() {
        assert_eq!(
            PathBuf::from("/foo/bar/baz.txt"),
            join_path("/foo", "bar/baz.txt")
        );
        assert_eq!(
            PathBuf::from("/foo/bar/baz.txt"),
            join_path(PathBuf::from("/foo/bar"), PathBuf::from("baz.txt"))
        );
    }
}
