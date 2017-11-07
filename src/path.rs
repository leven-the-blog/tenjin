const SEP: char = '.';

// Path

#[derive(Clone, Copy)]
pub enum Path<'a> {
    End,
    Pair(&'a str, &'a Path<'a>),
}

impl<'a> Path<'a> {
    pub fn new(s: &str) -> Path {
        Path::Pair(s, &Path::End)
    }

    pub fn parts(&self) -> Parts {
        Parts { path: *self }
    }

    pub fn prepend(&'a self, first: &'a str) -> Path<'a> {
        Path::Pair(first, self)
    }

    pub fn deconstruct(&self) -> Option<(&'a str, Path<'a>)> {
        let mut node = self;

        while let &Path::Pair(first, rest) = node {
            let mut chars  = first.chars();
            let mut substr = first;

            while let Some(c) = chars.next() {
                if c == SEP {
                    substr = chars.as_str();
                    continue;
                }

                return Some(
                    if let Some(end) = substr.find(SEP) {
                        let first  = &substr[..end];
                        let second = &substr[end + SEP.len_utf8()..];
                        (first, Path::Pair(second, rest))
                    } else {
                        (substr, *rest)
                    }
                );
            }

            node = rest;
        }

        None
    }

    pub fn to_owned(&self) -> PathBuf {
        let mut buf = PathBuf::new();
        let mut cur = self;

        if let &Path::Pair(first, next) = cur {
            buf.push_str(first);
            cur = next;
        } else {
            return buf;
        }

        while let &Path::Pair(first, next) = cur {
            buf.push(SEP);
            buf.push_str(first);
            cur = next;
        }

        buf
    }
}

// Parts

pub struct Parts<'a> {
    path: Path<'a>,
}

impl<'a> Iterator for Parts<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let (res, path) = match self.path.deconstruct() {
            Some((res, path)) => (Some(res), path),
            None => (None, Path::End),
        };

        self.path = path;
        res
    }
}

impl<'a> Parts<'a> {
    pub fn as_path(&self) -> Path {
        self.path
    }
}

// PathBuf

pub type PathBuf = String;
