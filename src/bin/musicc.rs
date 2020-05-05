//! `musicc` - pronounced *music-c*, is the compiler for syntxt files to wav files.

use std::env;
use std::io;
use std::path::PathBuf;

use syn_txt::declare_extension_value;
use syn_txt::lang::interpreter::{
    ArgParser, ExtensionValue, FromValue, Interpreter, InterpreterResult, IntpErr, IntpErrInfo,
    Value,
};
use syn_txt::lang::lexer::Lexer;
use syn_txt::lang::parser::Parser;
use syn_txt::lang::span::{LineMap, Span};
use syn_txt::note::{Note, NoteAction, Velocity};
use syn_txt::output;
use syn_txt::pianoroll::{PianoRoll, Time};
use syn_txt::rational::Rational;
use syn_txt::render;
use syn_txt::synth;
use syn_txt::wave;

fn main() -> io::Result<()> {
    let mut args = env::args_os().skip(1); // skip the program name
    let filename_os = args.next().expect("Usage: musicc <PATH>");
    let filename = PathBuf::from(filename_os);

    println!("Reading {}", filename.display());
    let source = std::fs::read_to_string(&filename)?;
    let roll = compile(&source)?;
    play(roll)
}

fn compile(input: &str) -> io::Result<PianoRoll> {
    let mut lex = Lexer::new(input);
    let mut tokens = Vec::new();
    let lines = LineMap::new(input);

    let mut has_errors = false;

    println!("Lexing...");
    while let Some(token_or_error) = lex.next_token() {
        match token_or_error {
            Ok(tok) => tokens.push(tok),
            Err(err) => {
                print_error(&lines, err.location(), err.kind());
                has_errors = true;
            }
        }
    }
    if has_errors {
        return Err(io::ErrorKind::InvalidData.into());
    }

    println!("Parsing...");
    let mut parser = Parser::new(input, &tokens);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(err) => {
            print_error(&lines, err.location(), err.info());
            return Err(io::ErrorKind::InvalidData.into());
        }
    };

    println!("Evaluating...");
    let mut int = Interpreter::new();
    int.register_primop("note", NoteValue::prim_new).unwrap();
    int.register_primop("piano-roll", PianoRollValue::prim_new)
        .unwrap();

    let mut final_value = Value::Unit;
    for s in ast {
        match int.eval(&s) {
            Ok(val) => final_value = val,
            Err(err) => {
                print_error(&lines, err.location(), err.info());
                return Err(io::ErrorKind::InvalidData.into());
            }
        }
    }

    PianoRollValue::from_value(final_value)
        .map(|val| val.0)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "not a piano roll"))
}

fn play(roll: PianoRoll) -> io::Result<()> {
    let bpm = 120;
    let measure = 4;
    let sample_rate = 44100;
    let time_to_sample = |time: Time| {
        (time.numerator() * measure * 60 * sample_rate / time.denominator() / bpm) as usize
    };

    let mut events = Vec::new();

    for pn in roll.iter() {
        let note = pn.note;
        let velocity = pn.velocity;
        let play = synth::Event {
            time: time_to_sample(pn.start),
            action: NoteAction::Play { note, velocity },
        };
        let release = synth::Event {
            time: time_to_sample(pn.start + pn.duration),
            action: NoteAction::Release { note },
        };

        events.push(play);
        events.push(release);
    }

    events.sort_by_key(|evt| evt.time);

    for evt in events.iter() {
        println!("{:?}", evt);
    }

    let max_samples = time_to_sample(roll.length()) + sample_rate as usize;

    let buffer_size = 441;

    let synth = synth::test::TestSynth::new(0, sample_rate as f64);
    let mut player = render::SynthPlayer::new(synth, &events);

    syn_txt::output::sox::with_sox_player(sample_rate as i32, |audio_stream| {
        let mut audio_buffer: Vec<wave::Stereo<f64>> = vec![
            wave::Stereo {
                left: 0.0,
                right: 0.0
            };
            buffer_size
        ];
        let mut byte_buffer = vec![0u8; buffer_size * 2 * 8];

        let mut samples_total = 0;
        while samples_total < max_samples {
            audio_buffer
                .iter_mut()
                .for_each(|s| *s = wave::Stereo::new(0.0, 0.0));
            player.generate(&mut audio_buffer);
            let n = output::copy_f64_bytes(&audio_buffer, &mut byte_buffer);
            assert_eq!(n, audio_buffer.len());
            audio_stream.write_all(&byte_buffer)?;
            samples_total += audio_buffer.len();
        }
        Ok(())
    })
}

fn print_error<E: std::fmt::Display>(lines: &LineMap, location: Span, message: E) {
    let start = lines.offset_to_pos(location.begin);
    let end = lines.offset_to_pos(location.end);
    println!("error: {} (<input>:{}-{})", message, start, end);
    println!("{}", lines.highlight(start, end, true));
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoteValue {
    pitch: Note,
    length: Rational,
    velocity: f64,
    offset: Option<Rational>,
}

impl NoteValue {
    fn prim_new(intp: &mut Interpreter, mut args: ArgParser) -> InterpreterResult<Value> {
        // mandatory arguments
        let mut pitch = None;
        let mut length = None;
        // optional arguments
        let mut velocity = 1.0;
        let mut offset = None;

        while !args.is_empty() {
            let key = args.keyword()?;
            match key.0.as_str() {
                ":pitch" => {
                    let name: String = args.extract(intp)?;
                    let named = Note::named_str(&name).ok_or_else(|| {
                        IntpErr::new(
                            args.list_span(),
                            IntpErrInfo::Other(format!("invalid note pitch `{}`", name)),
                        )
                    })?;
                    pitch = Some(named);
                }
                ":length" => length = Some(args.extract(intp)?),
                ":velocity" => velocity = args.extract(intp)?,
                ":offset" => offset = Some(args.extract(intp)?),
                _ => {
                    return Err(IntpErr::new(
                        args.list_span(),
                        IntpErrInfo::UnknownKeyword(key.clone()),
                    ))
                }
            }
        }

        let note = NoteValue {
            pitch: pitch
                .ok_or_else(|| IntpErr::new(args.list_span(), IntpErrInfo::NotEnoughArguments))?,
            length: length
                .ok_or_else(|| IntpErr::new(args.list_span(), IntpErrInfo::NotEnoughArguments))?,
            velocity,
            offset,
        };
        Ok(Value::ext(note))
    }
}

declare_extension_value!(NoteValue);

/// Simply a collection of notes
#[derive(Debug, Clone, PartialEq)]
pub struct PianoRollValue(PianoRoll);

declare_extension_value!(PianoRollValue);

impl PianoRollValue {
    fn prim_new(intp: &mut Interpreter, mut args: ArgParser) -> InterpreterResult<Value> {
        let mut roll = PianoRoll::new();
        while !args.is_empty() {
            let value = args.value(intp)?;
            // Allow both notes and other piano rolls when constructing new piano rolls
            match NoteValue::from_value(value) {
                Ok(note) => {
                    if let Some(offset) = note.offset {
                        roll.add_stack_offset(
                            note.pitch,
                            note.length,
                            Velocity::from_f64(note.velocity),
                            offset,
                        )
                    } else {
                        roll.add_after(note.pitch, note.length, Velocity::from_f64(note.velocity))
                    }
                }
                Err(other) => match PianoRollValue::from_value(other) {
                    Ok(other_roll) => roll.append(&other_roll.0),
                    Err(_) => return Err(IntpErr::new(args.last_span(), IntpErrInfo::Type)),
                },
            }
        }
        Ok(Value::ext(PianoRollValue(roll)))
    }
}
