// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

//! Converting between `Value` and Rust values.

use super::{Gc, GcPin, Symbol, Value};
use crate::rational::Rational;
use std::collections::HashMap;

pub trait ParseValue {
    type Repr;

    // TODO: add context for better error messages
    fn parse(&self, value: GcPin<Value>) -> Option<Self::Repr>;

    fn map<R, F: Fn(Self::Repr) -> R>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
    {
        Map {
            fun: f,
            parser: self,
        }
    }

    fn and_then<R, F: Fn(Self::Repr) -> Option<R>>(self, f: F) -> AndThen<Self, F>
    where
        Self: Sized,
    {
        AndThen {
            fun: f,
            parser: self,
        }
    }

    fn or<Q: ParseValue<Repr = Self::Repr>>(self, other: Q) -> Or<Self, Q>
    where
        Self: Sized,
    {
        Or {
            first: self,
            second: other,
        }
    }
}

impl<P: ParseValue> ParseValue for &P {
    type Repr = P::Repr;

    fn parse(&self, value: GcPin<Value>) -> Option<Self::Repr> {
        (*self).parse(value)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PrimParser<T>(fn(value: GcPin<Value>) -> Option<T>);

impl<T> ParseValue for PrimParser<T> {
    type Repr = T;

    fn parse(&self, value: GcPin<Value>) -> Option<Self::Repr> {
        self.0(value)
    }
}

macro_rules! prim_parser_impl {
    ($name:ident : $con:ident => $prim:ty) => {
        pub fn $name() -> PrimParser<$prim> {
            fn extract(value: GcPin<Value>) -> Option<$prim> {
                match &*value {
                    Value::$con(x) => Some(x.clone()),
                    _ => None,
                }
            }
            PrimParser(extract)
        }
    };
}

prim_parser_impl!(int : Int => i64);
prim_parser_impl!(ratio : Ratio => Rational);
prim_parser_impl!(float : Float => f64);
prim_parser_impl!(string : Str => String);
prim_parser_impl!(bool : Bool => bool);
prim_parser_impl!(dict : Dict => HashMap<Symbol, Gc<Value>>);

pub fn float_coercing() -> impl ParseValue<Repr = f64> {
    float()
        .or(int().map(|i| i as f64))
        .or(ratio().map(|r| r.numerator() as f64 / r.denominator() as f64))
}

pub fn ratio_coercing() -> impl ParseValue<Repr = Rational> {
    ratio().or(int().map(Rational::from_int))
}

pub struct ListParser<I> {
    item_parser: I,
}

impl<I: ParseValue> ParseValue for ListParser<I> {
    type Repr = Vec<I::Repr>;

    fn parse(&self, value: GcPin<Value>) -> Option<Self::Repr> {
        let mut out = Vec::new();
        let mut current = value;
        loop {
            match &*current {
                Value::Cons(head, tail) => {
                    out.push(self.item_parser.parse(head.pin())?);
                    current = tail.pin();
                }
                Value::Nil => break,
                _ => return None,
            }
        }
        Some(out)
    }
}

pub fn list<I: ParseValue>(item_parser: I) -> ListParser<I> {
    ListParser { item_parser }
}

pub struct RecordParser<F> {
    type_id: Option<String>,
    field_parser: F,
}

pub struct RecordFields {
    fields: HashMap<Symbol, Gc<Value>>,
}

impl RecordFields {
    pub fn get<P: ParseValue>(&self, key: &str, parser: P) -> Option<P::Repr> {
        self.fields.get(key).and_then(|x| parser.parse(x.pin()))
    }

    pub fn get_or<P: ParseValue>(&self, key: &str, default: P::Repr, parser: P) -> Option<P::Repr> {
        if let Some(value) = self.fields.get(key) {
            // the default only applies when the key is not there, not on parse failure
            parser.parse(value.pin())
        } else {
            Some(default)
        }
    }
}

impl<R, F: Fn(RecordFields) -> Option<R>> ParseValue for RecordParser<F> {
    type Repr = R;

    fn parse(&self, value: GcPin<Value>) -> Option<Self::Repr> {
        let fields = RecordFields {
            fields: dict().parse(value)?,
        };
        let record_type = fields.get(":__type__", string())?;
        if self
            .type_id
            .as_deref()
            .map_or(false, |tid| record_type != tid)
        {
            return None;
        }
        let parser = &self.field_parser;
        parser(fields)
    }
}

pub fn record<R, F: Fn(RecordFields) -> Option<R>>(
    type_id: &str,
    field_parser: F,
) -> RecordParser<F> {
    RecordParser {
        type_id: Some(type_id.to_string()),
        field_parser,
    }
}

pub fn typed_dict<R, F: Fn(RecordFields) -> Option<R>>(field_parser: F) -> RecordParser<F> {
    RecordParser {
        type_id: None,
        field_parser,
    }
}

pub struct Map<P, F> {
    parser: P,
    fun: F,
}

impl<P: ParseValue, R, F: Fn(P::Repr) -> R> ParseValue for Map<P, F> {
    type Repr = R;

    fn parse(&self, value: GcPin<Value>) -> Option<Self::Repr> {
        self.parser.parse(value).map(&self.fun)
    }
}

pub struct AndThen<P, F> {
    parser: P,
    fun: F,
}

impl<P: ParseValue, R, F: Fn(P::Repr) -> Option<R>> ParseValue for AndThen<P, F> {
    type Repr = R;

    fn parse(&self, value: GcPin<Value>) -> Option<Self::Repr> {
        self.parser.parse(value).and_then(&self.fun)
    }
}

pub struct Or<P, Q> {
    first: P,
    second: Q,
}

impl<P: ParseValue, Q: ParseValue<Repr = P::Repr>> ParseValue for Or<P, Q> {
    type Repr = P::Repr;

    fn parse(&self, value: GcPin<Value>) -> Option<Self::Repr> {
        self.first
            .parse(GcPin::clone(&value))
            .or_else(|| self.second.parse(value))
    }
}
