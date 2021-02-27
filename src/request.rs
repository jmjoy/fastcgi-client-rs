use crate::Params;
use tokio::io::AsyncRead;

pub struct Request<'a, I: AsyncRead + Unpin> {
    pub(crate) params: Params<'a>,
    pub(crate) stdin: I,
}

impl<'a, I: AsyncRead + Unpin> Request<'a, I> {
    pub fn new(params: Params<'a>, stdin: I) -> Self {
        Self { params, stdin }
    }

    pub fn from_http_request(http_request: http::Request<I>) -> Self {
        // TODO fill logic
        Self {
            params: Default::default(),
            stdin: http_request.into_body(),
        }
    }

    pub fn params(&self) -> &Params<'a> {
        &self.params
    }

    pub fn params_mut(&mut self) -> &mut Params<'a> {
        &mut self.params
    }

    pub fn stdin(&self) -> &I {
        &self.stdin
    }

    pub fn stdin_mut(&mut self) -> &mut I {
        &mut self.stdin
    }
}
