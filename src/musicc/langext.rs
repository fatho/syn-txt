//! Music-specific language extensions for the underlying scheme-like language.

use crate::declare_extension_value;
use crate::lang::interpreter::{
    ArgParser, ExtensionValue, FromValue, Interpreter, InterpreterResult, IntpErr, IntpErrInfo,
    PrimOp, Value,
};
use crate::note::{Note, Velocity};
use crate::pianoroll::{PianoRoll, PlayedNote};
use crate::rational::Rational;
use std::iter::FromIterator;

pub static PRIMOPS: &[(&str, PrimOp)] = &[
    // Note type
    ("note", PrimOp(NoteValue::prim_new)),
    // PianoRoll type
    ("piano-roll", PrimOp(PianoRollValue::prim_new)),
    ("piano-roll/stack", PrimOp(PianoRollValue::prim_stack)),
];

#[derive(Debug, Clone, PartialEq)]
pub struct NoteValue {
    pub pitch: Note,
    pub length: Rational,
    pub velocity: f64,
    pub offset: Option<Rational>,
}

impl NoteValue {
    pub fn prim_new(intp: &mut Interpreter, mut args: ArgParser) -> InterpreterResult<Value> {
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
pub struct PianoRollValue(pub PianoRoll);

declare_extension_value!(PianoRollValue);

impl PianoRollValue {
    pub fn prim_new(intp: &mut Interpreter, mut args: ArgParser) -> InterpreterResult<Value> {
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

    /// Stack multiple piano rolls on top of each other, resulting in them all playing at once.
    pub fn prim_stack(intp: &mut Interpreter, mut args: ArgParser) -> InterpreterResult<Value> {
        let mut notes: Vec<PlayedNote> = Vec::new();

        while !args.is_empty() {
            let value = args.value(intp)?;
            // Allow both notes and other piano rolls when constructing new piano rolls
            match NoteValue::from_value(value) {
                Ok(note) => notes.push(PlayedNote {
                    note: note.pitch,
                    velocity: Velocity::from_f64(note.velocity),
                    start: note.offset.unwrap_or(Rational::ZERO),
                    duration: note.length,
                }),
                Err(other) => match PianoRollValue::from_value(other) {
                    Ok(other_roll) => notes.extend(other_roll.0.iter().cloned()),
                    Err(_) => return Err(IntpErr::new(args.last_span(), IntpErrInfo::Type)),
                },
            }
        }
        Ok(Value::ext(PianoRollValue(PianoRoll::from_iter(notes))))
    }
}
