use syntxt_lang::{eval, parser::Parser};


fn main() {
    let filename = std::env::args_os().skip(1).next().expect("Usage: syntxt-debug <FILE>");
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
