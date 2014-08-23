/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_name = "macros"]
#![crate_type = "rlib"]
#![crate_type = "dylib"]

#![feature(macro_rules, plugin_registrar)]

//! Exports macros for use in other Servo crates.

#[cfg(test)]
extern crate sync;

extern crate rustc;
extern crate syntax;
 
use syntax::ast;
use syntax::codemap::Span;
use syntax::ext::base;
use syntax::ext::base::{ExtCtxt, MacResult, MacPat};
use syntax::ext::build::AstBuilder;
use syntax::parse::token;
use rustc::plugin::Registry;
use std::gc::GC;


#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("ident_to_css_name", expand_ident_to_css_name)
}

fn expand_ident_to_css_name(cx: &mut ExtCtxt, sp: Span, tts: &[ast::TokenTree])
                            -> Box<base::MacResult> {
    let mut tts = tts.iter();
    match tts.next() {
        Some(&ast::TTTok(_, token::IDENT(ident, _))) => {
            if tts.next().is_none() {
                let new_value = token::get_ident(ident).get().replace("_", "-");
                let new_ident = token::intern_and_get_ident(new_value.as_slice());

                MacPat::new(box(GC) ast::Pat {
                    id: ast::DUMMY_NODE_ID,
                    node: ast::PatLit(cx.expr_str(sp, new_ident)),
                    span: sp,
                })
            } else {
                cx.span_err(sp, "Expected 1 argument, found more");
                base::DummyResult::any(sp)
            }
        }
        _ => {
            cx.span_err(sp, "Expected ident argument");
            base::DummyResult::any(sp)
        }
    }
}


#[macro_export]
macro_rules! bitfield(
    ($bitfieldname:ident, $getter:ident, $setter:ident, $value:expr) => (
        impl $bitfieldname {
            #[inline]
            pub fn $getter(self) -> bool {
                let $bitfieldname(this) = self;
                (this & $value) != 0
            }

            #[inline]
            pub fn $setter(&mut self, value: bool) {
                let $bitfieldname(this) = *self;
                *self = $bitfieldname((this & !$value) | (if value { $value } else { 0 }))
            }
        }
    )
)


#[macro_export]
macro_rules! lazy_init(
    ($(static ref $N:ident : $T:ty = $e:expr;)*) => (
        $(
            #[allow(non_camel_case_types)]
            struct $N {__unit__: ()}
            static $N: $N = $N {__unit__: ()};
            impl Deref<$T> for $N {
                fn deref<'a>(&'a self) -> &'a $T {
                    unsafe {
                        static mut s: *const $T = 0 as *const $T;
                        static mut ONCE: ::sync::one::Once = ::sync::one::ONCE_INIT;
                        ONCE.doit(|| {
                            s = ::std::mem::transmute::<Box<$T>, *const $T>(box () ($e));
                        });
                        &*s
                    }
                }
            }

        )*
    )
)


#[cfg(test)]
mod tests {
    use std::collections::hashmap::HashMap;
    lazy_init! {
        static ref NUMBER: uint = times_two(3);
        static ref VEC: [Box<uint>, ..3] = [box 1, box 2, box 3];
        static ref OWNED_STRING: String = "hello".to_string();
        static ref HASHMAP: HashMap<uint, &'static str> = {
            let mut m = HashMap::new();
            m.insert(0u, "abc");
            m.insert(1, "def");
            m.insert(2, "ghi");
            m
        };
    }

    fn times_two(n: uint) -> uint {
        n * 2
    }

    #[test]
    fn test_basic() {
        assert_eq!(*OWNED_STRING, "hello".to_string());
        assert_eq!(*NUMBER, 6);
        assert!(HASHMAP.find(&1).is_some());
        assert!(HASHMAP.find(&3).is_none());
        assert_eq!(VEC.as_slice(), &[box 1, box 2, box 3]);
    }

    #[test]
    fn test_repeat() {
        assert_eq!(*NUMBER, 6);
        assert_eq!(*NUMBER, 6);
        assert_eq!(*NUMBER, 6);
    }
}
