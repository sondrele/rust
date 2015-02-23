// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub struct Foo<T>(T, T);

impl<T> Foo<T> {
    fn foo(&self) {
        match *self {
            // The error message is not ideal in this case, can we do
            // better here?
            Foo<T>(ref x, ref y) => {}  //~ ERROR expected identifier, found enum or struct
        }
    }
}

fn main() {}
