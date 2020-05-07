// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

//! Music-specific language extensions for the underlying scheme-like language.

use crate::declare_extension_value;
use crate::lang::interpreter::{
    ArgParser, ExtensionValue, FromValue, Interpreter, InterpreterResult, IntpErr, IntpErrInfo,
    PrimOp, Value,
};
use crate::note::{Note, Velocity};
use crate::pianoroll::{PianoRoll, PlayedNote};
use crate::rational::Rational;
use std::{iter::FromIterator, rc::Rc};

pub static PRIMOPS: &[(&str, PrimOp)] = &[
    // `NotePitch` type
    ("transpose", PrimOp(NotePitch::prim_transpose)),
    // `Note` type
    ("note", PrimOp(NoteValue::prim_new)),
    // `PianoRoll` type
    ("piano-roll", PrimOp(PianoRollValue::prim_new)),
    ("piano-roll/stack", PrimOp(PianoRollValue::prim_stack)),
    // `Song` type
    ("song", PrimOp(SongValue::prim_new)),
];

/// A macro for facilitating the parsing of prim-op args
macro_rules! parse_primop_args {
    (__make_let $field:ident : $field_ty:ty = mandatory) => { let mut $field: Option<$field_ty> = None; };
    (__make_let $field:ident : $field_ty:ty = optional) => { let mut $field: Option<$field_ty> = None; };
    (__make_let $field:ident : $field_ty:ty = $val:expr) => { let mut $field: $field_ty = $val; };

    (__make_pat ($args:expr, $intp:expr) $key:expr => $field:ident = mandatory) => { Some($args.extract($intp)?) };
    (__make_pat ($args:expr, $intp:expr) $key:expr => $field:ident = optional) => { Some($args.extract($intp)?) };
    (__make_pat ($args:expr, $intp:expr) $key:expr => $field:ident = $val:expr) => { $args.extract($intp)? };

    (__make_assign($args:expr) $field:ident = mandatory) => {
        $field.ok_or_else(|| IntpErr::new($args.list_span(), IntpErrInfo::NotEnoughArguments))?.into()
    };
    (__make_assign($args:expr) $field:ident = optional) => { $field.into() };
    (__make_assign($args:expr) $field:ident = $val:expr) => { $field.into() };

    ($intp:expr, $args:expr, { $($key:expr => $field:ident : $field_ty:ty = $def:tt),* , }) => {
        $(parse_primop_args!(__make_let $field : $field_ty = $def));+ ;

        while !$args.is_empty() {
            let key = $args.keyword()?;
            match key.0.as_ref() {
                $($key => $field = parse_primop_args!(__make_pat($args, $intp) $key => $field = $def)),* ,
                _ => {
                    return Err(IntpErr::new(
                        $args.list_span(),
                        IntpErrInfo::UnknownKeyword(key.clone()),
                    ))
                }
            }
        }
    };
}

/// Define a new record value in the language.
macro_rules! langext_record_value {
    (__make_assign($args:expr) $field:ident = mandatory) => {
        $field.ok_or_else(|| IntpErr::new($args.list_span(), IntpErrInfo::NotEnoughArguments))?.into()
    };
    (__make_assign($args:expr) $field:ident = optional) => { $field.into() };
    (__make_assign($args:expr) $field:ident = $val:expr) => { $field.into() };

    ($name:ident { $($key:expr => $field:ident : $field_ty:ty = $def:tt),* , }) => {
        impl $name {
            pub fn prim_new(intp: &mut Interpreter, mut args: ArgParser) -> InterpreterResult<Value> {
                parse_primop_args!(intp, args, { $($key => $field : $field_ty = $def),* ,} );
                let val = $name {
                    $($field: langext_record_value!(__make_assign(args) $field = $def)),*,
                };
                Ok(Value::ext(val))
            }
        }

        declare_extension_value!($name);
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoteValue {
    pub pitch: Note,
    pub length: Rational,
    pub velocity: f64,
    pub offset: Option<Rational>,
}

/// Helper struct for parsing `Note`s from values.
struct NotePitch(Note);

impl From<NotePitch> for Note {
    fn from(pitch: NotePitch) -> Self {
        pitch.0
    }
}

impl FromValue for NotePitch {
    fn from_value(value: Value) -> Result<Self, Value> {
        // Allow both names and Midi indexes
        // Try int first, because otherwise the int gets converted to a string
        log::trace!("NotePitch::from_value({:?})", value);
        i64::from_value(value)
            .and_then(|midi| {
                Note::try_from_midi(midi)
                    .map(NotePitch)
                    .ok_or(Value::Int(midi))
            })
            .or_else(|value| {
                String::from_value(value).and_then(|name| {
                    log::trace!("NotePitch::from_value({})", name);
                    if let Some(note) = Note::named_str(&name) {
                        Ok(NotePitch(note))
                    } else {
                        Err(Value::Str(name.into()))
                    }
                })
            })
    }
}

impl NotePitch {
    fn to_value(self) -> Value {
        Value::Int(self.0.to_midi() as i64)
    }

    pub fn prim_transpose(intp: &mut Interpreter, mut args: ArgParser) -> InterpreterResult<Value> {
        let note = args.extract::<NotePitch>(intp)?;
        let amount = args.extract::<i64>(intp)?;
        if let Some(new_note) = Note::try_from_midi(note.0.to_midi() as i64 + amount) {
            Ok(NotePitch(new_note).to_value())
        } else {
            Err(IntpErr::new(
                args.list_span(),
                IntpErrInfo::Other(format!("transposed pitch exceeds MIDI range")),
            ))
        }
    }
}

langext_record_value! {
    NoteValue {
        ":pitch" => pitch: NotePitch = mandatory,
        ":length" => length: Rational = mandatory,
        ":velocity" => velocity: f64 = 1.0,
        ":offset" => offset: Rational = optional,
    }
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct SongValue {
    pub bpm: i64,
    pub notes: Rc<PianoRollValue>,
}

langext_record_value! {
    SongValue {
        ":bpm" => bpm: i64 = mandatory,
        ":notes" => notes: Rc<PianoRollValue> = mandatory,
    }
}
