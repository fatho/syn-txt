// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2021  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use syntxt_lang::{eval, parser::Parser};

fn main() {
    let filename = std::env::args_os()
        .skip(1)
        .next()
        .expect("Usage: syntxt-debug <FILE>");
    let source = std::fs::read_to_string(filename).unwrap();
    let ast = match Parser::parse(&source) {
        Ok(ast) => ast,
        Err((partial_ast, errors)) => {
            for err in errors {
                println!("[ERR] {} @ {}", err.message, err.pos.start);
            }
            partial_ast
        }
    };
    println!("{:#?}", ast);

    let mut cxt = eval::Context::new();
    let objects = cxt.eval(&ast);
    println!("Root Objects: {:?}", objects);
    println!("Context:\n{:#?}", cxt);
}
