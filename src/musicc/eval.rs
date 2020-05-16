// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

//! The module responsible for evaluating syn.txt files to an abstract description of a song.

use log::error;
use std::io;

use crate::lang::compiler;
use crate::lang::debug::{DebugTable, SourceLocation};
use crate::lang::heap::{GcPin, Heap};
use crate::lang::interpreter::Interpreter;
use crate::lang::marshal;
use crate::lang::pretty::pretty;
use crate::lang::span::LineMap;
use crate::lang::value::Value;

use super::{langext, song};

use std::path::Path;

/// Evaluate syn.txt source code into a song description.
///
/// TODO: allow including other files.
pub fn eval(input_name: &str, input: &str, dump_value: Option<&Path>) -> io::Result<song::Song> {
    let mut heap = Heap::new();
    let mut debug = DebugTable::new();
    let values = compiler::compile_str(&mut heap, &mut debug, input_name, input).unwrap();

    log::info!("evaluating {}", input_name);
    let mut int = Interpreter::new(&mut heap, &mut debug);
    for (name, op) in langext::PRIMOPS {
        int.register_primop(name, op.0)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}", e.info())))?;
    }

    static MUSIC_PRELUDE: &str = include_str!("Music.syn");
    int.source_prelude("<music-prelude>", MUSIC_PRELUDE)
        .expect("music prelude should compile");

    let mut final_value = int.heap_alloc_value(Value::Void).pin();
    for v in values {
        match int.eval(v) {
            Ok(val) => final_value = val.pin(),
            Err(err) => {
                let source = err
                    .location()
                    .and_then(|loc| int.debug_info().get_source(&loc.file));
                let lines = source.map(LineMap::new);
                log_error(lines.as_ref(), err.location(), err.info());
                return Err(io::Error::new(io::ErrorKind::InvalidData, err));
            }
        }
    }

    drop(int);
    heap.gc_cycles();

    if let Some(dump_out) = dump_value {
        use io::Write;
        let mut outfile = std::fs::File::create(dump_out)?;
        writeln!(&mut outfile, "{}", pretty(&final_value))?;
        drop(outfile);
    }

    build_song(final_value).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "not a song"))
}

fn build_song(value: GcPin<Value>) -> Option<song::Song> {
    use marshal::ParseValue;
    parsers::song().parse(value)
}

fn log_error<E: std::fmt::Display>(
    lines: Option<&LineMap>,
    location: Option<&SourceLocation>,
    message: E,
) {
    use std::fmt::Write;
    let mut buf = String::new();
    write!(&mut buf, "error: {}", message).unwrap();
    if let Some(location) = location {
        if let Some(lines) = lines {
            let start = lines.offset_to_pos(location.span.begin);
            let end = lines.offset_to_pos(location.span.end);
            writeln!(&mut buf, "({} {}-{})", location.file, start, end).unwrap();
            writeln!(&mut buf, "{}", lines.highlight(start, end, true)).unwrap();
        } else {
            writeln!(
                &mut buf,
                "({} {}-{})",
                location.file, location.span.begin, location.span.end
            )
            .unwrap();
        }
    }
    error!("{}", buf);
}

// ============================ VALUE PARSERS ============================

pub mod parsers {
    use super::{langext, song};
    use crate::lang::{
        marshal::{self, ParseValue},
        Gc, Value,
    };
    use crate::note::Velocity;
    use crate::pianoroll::{PianoRoll, PlayedNote};
    use crate::synth;

    fn update_if_valid<T, P: ParseValue<Repr = T>>(
        target: &mut T,
        value: Gc<Value>,
        context: &str,
        parser: P,
    ) {
        if let Some(value) = parser.parse(value.pin()) {
            *target = value;
        } else {
            log::warn!("ignoring invalid value for {}", context)
        }
    }

    pub fn wave_shape() -> impl marshal::ParseValue<Repr = synth::oscillator::WaveShape> {
        marshal::string().and_then(|mut name| {
            name.make_ascii_lowercase();
            match name.as_str() {
                "sine" => Some(synth::oscillator::WaveShape::Sine),
                "saw" => Some(synth::oscillator::WaveShape::Saw),
                "supersaw" => Some(synth::oscillator::WaveShape::SuperSaw),
                "twosidedsaw" => Some(synth::oscillator::WaveShape::TwoSidedSaw),
                "alternatingsaw" => Some(synth::oscillator::WaveShape::AlternatingSaw),
                other => {
                    log::error!("unknown wave shape {}", other);
                    None
                }
            }
        })
    }

    pub fn test_synth_params() -> impl marshal::ParseValue<Repr = synth::test::Params> {
        marshal::dict().and_then(|dict| {
            let mut params = synth::test::Params::default();
            for (key, value) in dict.into_iter() {
                match key.as_str() {
                    ":gain" => update_if_valid(
                        &mut params.gain,
                        value,
                        "test.gain",
                        marshal::float_coercing(),
                    ),
                    ":pan" => update_if_valid(
                        &mut params.pan,
                        value,
                        "test.pan",
                        marshal::float_coercing(),
                    ),
                    ":unison" => update_if_valid(
                        &mut params.unison,
                        value,
                        "test.unison",
                        marshal::int().map(|i| i as usize),
                    ),
                    ":unison-detune" => update_if_valid(
                        &mut params.unison_detune_cents,
                        value,
                        "test.unison-detune",
                        marshal::float_coercing(),
                    ),
                    ":unison-falloff" => update_if_valid(
                        &mut params.unison_falloff,
                        value,
                        "test.unison-falloff",
                        marshal::float_coercing(),
                    ),
                    ":wave-shape" => update_if_valid(
                        &mut params.wave_shape,
                        value,
                        "test.wave-shape",
                        wave_shape(),
                    ),
                    ":envelope" => update_if_valid(
                        &mut params.envelope,
                        value,
                        "test.envelope",
                        envelope(),
                    ),
                    ":filter" => update_if_valid(
                        &mut params.filter,
                        value,
                        "test.filter",
                        biquad_filter(),
                    ),
                    other => log::warn!("unused test synth parameter {}", other),
                }
            }
            Some(params)
        })
    }

    pub fn synth() -> impl marshal::ParseValue<Repr = song::Instrument> {
        marshal::record("synth", |fields| {
            let name = fields.get(":name", marshal::string())?;
            match name.as_ref() {
                "test" => fields
                    .get(":params", test_synth_params())
                    .map(song::Instrument::TestSynth),
                _ => {
                    log::error!("unknown synth {:?}", name);
                    None
                }
            }
        })
    }

    pub fn envelope() -> impl marshal::ParseValue<Repr = synth::envelope::ADSR> {
        marshal::record("asdr", |fields| {
            Some(synth::envelope::ADSR {
                attack: fields.get(":attack", marshal::float_coercing())?,
                decay: fields.get(":decay", marshal::float_coercing())?,
                sustain: fields.get(":sustain", marshal::float_coercing())?,
                release: fields.get(":release", marshal::float_coercing())?,
            })
        })
    }

    pub fn biquad_filter() -> impl marshal::ParseValue<Repr = synth::filter::BiquadType> {
        use synth::filter::BiquadType;
        marshal::record("biquad-filter", |fields| {
            let name = fields.get(":name", marshal::string())?;
            match name.as_ref() {
                "allpass" => Some(BiquadType::Allpass),
                "lowpass" => {
                    let cutoff = fields.get(":cutoff", marshal::float_coercing())?;
                    let q = fields.get(":q", marshal::float_coercing())?;
                    Some(BiquadType::Lowpass { cutoff, q })
                },
                _ => {
                    log::error!("unknown synth {:?}", name);
                    None
                }
            }
        })
    }

    pub fn instrument() -> impl marshal::ParseValue<Repr = song::Instrument> {
        synth()
    }

    pub fn song() -> impl marshal::ParseValue<Repr = song::Song> {
        let note_parser = marshal::record("note", |fields| {
            Some(PlayedNote {
                note: fields.get(":pitch", langext::note_parser())?,
                velocity: fields.get_or(
                    ":velocity",
                    Velocity::MAX,
                    marshal::float_coercing().and_then(Velocity::try_from_f64),
                )?,
                start: fields.get(":start", marshal::ratio_coercing())?,
                duration: fields.get(":length", marshal::ratio_coercing())?,
            })
        });
        let note_list_parser = marshal::list(note_parser);
        marshal::record("song", move |fields| {
            let bpm = fields.get(":bpm", marshal::int())?;
            let note_list = fields.get(":notes", &note_list_parser)?;
            let notes = Some(PianoRoll::with_notes(note_list))?;
            let instrument = fields.get(":instrument", instrument())?;
            Some(song::Song {
                bpm,
                notes,
                instrument,
            })
        })
    }
}
