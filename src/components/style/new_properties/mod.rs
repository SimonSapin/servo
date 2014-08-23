/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use serialize::{Encodable, Encoder};
use std::ascii::StrAsciiExt;
use sync::Arc;

use cssparser::*;
use cssparser::ast::*;
use url::Url;

use servo_util::geometry::Au;
use errors::{log_css_error, ErrorLoggerIterator};
use properties::common_types::specified;

use self::property_declaration::PropertyDeclaration;


mod longhand_definitions;
mod shorthand_definitions;


mod longhands {
    macro_rules! define_longhand_types {
        ( $( $name: ident {
            SpecifiedValue = $specified_value_type: path;
            $stuff: item
        } )+ ) => {
            $(
                pub mod $name {
                    #[allow(unused_imports)]
                    use properties::common_types::specified;
                    pub type SpecifiedValue = $specified_value_type;
                    $stuff
                }
            )+
        }
    }
    with_longhand_definitions!(define_longhand_types)
}

mod property_declaration {
    use super::{DeclaredValue, longhands};

    macro_rules! define_property_declaration {
        ( $( $name: ident {
            SpecifiedValue = $specified_value_type: path;
            $stuff: item
        } )+ ) => {
            #[allow(non_camel_case_types)]
            #[deriving(Clone)]
            pub enum PropertyDeclaration {
                $(
                    $name(DeclaredValue<longhands::$name::SpecifiedValue>),
                )+
            }
        }
    }
    with_longhand_definitions!(define_property_declaration)
}


#[deriving(Clone)]
pub enum DeclaredValue<T> {
    SpecifiedValue(T),
    Initial,
    Inherit,
    // There is no Unset variant here.
    // The 'unset' keyword is represented as either Initial or Inherit,
    // depending on whether the property is inherited.
}


pub enum PropertyDeclarationParseResult {
    UnknownProperty,
    ExperimentalProperty,
    InvalidValue,
    ValidOrIgnoredDeclaration,
}


fn parse_property_declaration(name: &str, value: &[ComponentValue],
             result_list: &mut Vec<PropertyDeclaration>,
             base_url: &Url
             //seen: &mut PropertyBitField
             ) -> PropertyDeclarationParseResult {
    macro_rules! define_parse_shorthands {
        ( $(
            $name: ident
            |$value_arg: ident, $base_url_arg: ident|
            -> ( $($longhand: ident),+ )
            $parse: block
        )+ ) => {
            match name.to_ascii_lower().as_slice() {
                $(
                    ident_to_css_name!($name) => {
                        struct Longhands {
                            $(
                                $longhand: Option<longhands::$longhand::SpecifiedValue>,
                            )+
                        }
                        let $value_arg = value;
                        let $base_url_arg = base_url;
                        match $parse {
                            Ok(result) => {
                                $(
                                    result_list.push(property_declaration::$longhand(
                                        match result.$longhand {
                                            Some(value) => SpecifiedValue(value),
                                            None => Initial,
                                        }
                                    ));
                                )+
                                ValidOrIgnoredDeclaration
                            },
                            Err(()) => InvalidValue
                        }
                    }
                )+
                _ => UnknownProperty
            }
        }
    }
    with_shorthand_definitions!(define_parse_shorthands)
}


/// Declarations are stored in reverse order.
/// Overridden declarations are skipped.
pub struct PropertyDeclarationBlock {
    important: Arc<Vec<PropertyDeclaration>>,
    normal: Arc<Vec<PropertyDeclaration>>,
}

impl<E, S: Encoder<E>> Encodable<S, E> for PropertyDeclarationBlock {
    fn encode(&self, _: &mut S) -> Result<(), E> {
        Ok(())
    }
}


pub fn parse_style_attribute(input: &str, base_url: &Url) -> PropertyDeclarationBlock {
    parse_property_declaration_list(tokenize(input), base_url)
}


pub fn parse_property_declaration_list<I: Iterator<Node>>(input: I, base_url: &Url)
                                                          -> PropertyDeclarationBlock {
    let mut important_declarations = vec!();
    let mut normal_declarations = vec!();
//    let mut important_seen = PropertyBitField::new();
//    let mut normal_seen = PropertyBitField::new();
    let items: Vec<DeclarationListItem> =
        ErrorLoggerIterator(parse_declaration_list(input)).collect();
    for item in items.move_iter().rev() {
        match item {
            DeclAtRule(rule) => log_css_error(
                rule.location, format!("Unsupported at-rule in declaration list: @{:s}", rule.name).as_slice()),
            Declaration(Declaration{ location: l, name: n, value: v, important: i}) => {
                // TODO: only keep the last valid declaration for a given name.
//                let (list, seen) = if i {
//                    (&mut important_declarations, &mut important_seen)
//                } else {
//                    (&mut normal_declarations, &mut normal_seen)
//                };
                let list = if i {
                    &mut important_declarations
                } else {
                    &mut normal_declarations
                };
                match parse_property_declaration(n.as_slice(), v.as_slice(), list,
                                                 base_url/*, seen*/) {
                    UnknownProperty => log_css_error(l, format!(
                        "Unsupported property: {}:{}", n, v.iter().to_css()).as_slice()),
                    ExperimentalProperty => log_css_error(l, format!(
                        "Experimental property, use `servo --enable_experimental` \
                         or `servo -e` to enable: {}:{}",
                        n, v.iter().to_css()).as_slice()),
                    InvalidValue => log_css_error(l, format!(
                        "Invalid value: {}:{}", n, v.iter().to_css()).as_slice()),
                    ValidOrIgnoredDeclaration => (),
                }
            }
        }
    }
    PropertyDeclarationBlock {
        important: Arc::new(important_declarations),
        normal: Arc::new(normal_declarations),
    }
}

pub fn dummy(block: PropertyDeclarationBlock) {
    drop(block.normal);
    drop(block.important);
}
