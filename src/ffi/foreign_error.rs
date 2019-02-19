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

use std::{error::Error, fmt::{Display, Formatter, Result}};

use libc::c_int;

#[derive(Debug, Copy, Clone)]
pub struct ForeignError<'a> {
    code: c_int,
    description: &'a str,
}

impl<'a> ForeignError<'a> {
    pub fn new(code: c_int, description: &'a str) -> ForeignError<'a> {
        assert!(code != 0);

        ForeignError{ code, description }
    }
}

impl<'a> Error for ForeignError<'a> { }

impl<'a> Display for ForeignError<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{} ({})", self.description, self.code)
    }
}
