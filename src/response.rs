use std::{collections::HashMap, fmt, fmt::Debug};

pub(crate) type ResponseMap = HashMap<u16, Response>;

/// Output of fastcgi request, contains STDOUT and STDERR.
#[derive(Default, Clone)]
pub struct Response {
    stdout: Option<Vec<u8>>,
    stderr: Option<Vec<u8>>,
}

impl Response {
    pub(crate) fn set_stdout(&mut self, stdout: Vec<u8>) {
        match self.stdout {
            Some(ref mut buf) => buf.extend(stdout.iter()),
            None => self.stdout = Some(stdout),
        }
    }

    pub(crate) fn set_stderr(&mut self, stderr: Vec<u8>) {
        match self.stderr {
            Some(ref mut buf) => buf.extend(stderr.iter()),
            None => self.stderr = Some(stderr),
        }
    }

    pub fn get_stdout(&self) -> Option<Vec<u8>> {
        self.stdout.clone()
    }

    pub fn get_stderr(&self) -> Option<Vec<u8>> {
        self.stderr.clone()
    }
}

impl Debug for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        Debug::fmt(r#"Output { stdout: "...", stderr: "..." }"#, f)
    }
}
