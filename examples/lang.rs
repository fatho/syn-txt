use syn_txt::lang::lexer::Lexer;
use syn_txt::lang::parser::Parser;

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

    let mut lex = Lexer::new(input);
    let mut tokens = Vec::new();

    while let Some(tok) = lex.next_token() {
        if tok.1.is_error() {
            println!("ERROR {:?}: {:?}", tok.1, &input[tok.0.begin..tok.0.end]);
        } else {
            println!("{:?}: {:?} ({:?})", tok.1, &input[tok.0.begin..tok.0.end], tok.0);
            tokens.push(tok);
        }
    }

    let mut parser = Parser::new(input, &tokens);
    println!("{:?}", parser.parse());
}
