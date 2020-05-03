use syn_txt::lang::interpreter::Interpreter;
use syn_txt::lang::lexer::Lexer;
use syn_txt::lang::parser::Parser;
use syn_txt::lang::span::{LineMap, Span};

fn main() {
    let input = r#"
        (define lead-synth
            (instruments/the-synth
                :pan 0.0
                :gain 1.0
            )
        )

        (define
            (channel
                :gain 1.0
            )
        )

        ; This is a melody in A minor
        (define ascending-melody
            (piano-roll
                (note :pitch "a4" :length 1/4 :velocity 0.1)
                (note :pitch "b4" :length 1/4 :velocity 0.3)
                (note :pitch "c5" :length 1/4 :velocity 0.5)
                (note :pitch "d5" :length 1/4 :velocity 0.7)
                (note :pitch "e5" :length 1/1 :velocity 1.0)
                ; Note how these two notes don't start *after* the previous note ended,
                ; but only 1/4th after the previous note *started*,
                (note :pitch "a5" :length 3/4 :velocity 1.0 :offset: 1/4)
                (note :pitch "c5" :length 1/2 :velocity 1.0 :offset: 1/4)
            )
        )

        ; Play the above melody twice
        (define final-melody
            (sequence
                ascending-melody
                ascending-melody)
        )

        (song
            :bpm 128
            :measure 4
            :channels (channel-set "master" master)
            :tracks (track-set "lead" (track :instrument lead-synth))
            :playlist
                (playlist
                    (play "lead" final-melody)
                )
        )
    "#;

    let input = r#"
        (define u 1/13)
        (define x 1/4)
        (define y (* -2 x))
        (define z (/ 1
                     1))
        (define a (+ x y z))
        a"#;

    run_test(input)
}

fn run_test(input: &str) {
    let mut lex = Lexer::new(input);
    let mut tokens = Vec::new();
    let lines = LineMap::new(input);

    println!("Lexing...");
    while let Some(token_or_error) = lex.next_token() {
        match token_or_error {
            Ok(tok) => tokens.push(tok),
            Err(err) => {
                print_error(&lines, err.location(), err.kind());
            }
        }
    }

    println!("Parsing...");
    let mut parser = Parser::new(input, &tokens);
    // println!("{:?}", parser.parse());
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(err) => {
            print_error(&lines, err.location(), err.info());
            return;
        }
    };

    println!("Evaluating...");
    let mut int = Interpreter::new();
    for s in ast {
        println!("{}", &input[s.src.begin..s.src.end]);
        match int.eval(&s) {
            Ok(val) => println!("  {:?}", val),
            Err(err) => {
                print_error(&lines, err.location(), err.info());
                return;
            }
        }
    }
}

fn print_error<E: std::fmt::Display>(lines: &LineMap, location: Span, message: E) {
    let start = lines.offset_to_pos(location.begin);
    let end = lines.offset_to_pos(location.end);
    println!("error: {} (<input>:{}-{})", message, start, end);
    println!("{}", lines.highlight(start, end, true));
}
