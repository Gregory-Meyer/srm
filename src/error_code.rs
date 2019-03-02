// Copyright 2019 Gregory Meyer
//
// Permission is hereby granted, free of charge, to any person
// obtaining a copy of this software and associated documentation
// files (the "Software"), to deal in the Software without
// restriction, including without limitation the rights to use, copy,
// modify, merge, publish, distribute, sublicense, and/or sell copies
// of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS
// BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
// ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use super::*;

use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
};

use libc::c_int;

/// An error condition from outside of Rust.
///
/// # Examples
/// ```
/// unsafe fn open(pathname: &CStr) -> Result<c_int, ErrorCode<'static>> {
///     match libc::open(pathname.as_ptr(), O_CREAT | O_RDWR) {
///         -1 => {
///             let errno = *libc::__errno_location();
///             let msg = CStr::from_ptr(libc::strerror(errno)).to_str().unwrap();
///
///             Err(ErrorCode::new(errno, msg))
///         }
///         x => Ok(x)
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ErrorCode {
    code: c_int,
    description: String,
}

impl ErrorCode {
    /// Creates a new error code/description pair from its parts.
    ///
    /// `code` must be nonzero, as zero indicates success.
    pub fn new(code: c_int, description: String) -> ErrorCode {
        assert!(code != 0);

        ErrorCode { code, description }
    }
}

impl Error for ErrorCode {}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{} ({})", self.description, self.code)
    }
}
