/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![macro_escape]


macro_rules! four_sides {
    ($name_prefix: ident {} $name_suffix: ident $( $rest: tt )+ ) => {
        concat_ident!($name_prefix, top, $name_suffix) $( $rest )+
        concat_ident!($name_prefix, right, $name_suffix) $( $rest )+
        concat_ident!($name_prefix, bottom, $name_suffix) $( $rest )+
        concat_ident!($name_prefix, left, $name_suffix) $( $rest )+
    }
}


macro_rules! with_longhand_definitions { ($macro: ident) => { $macro! {

// Start of longhand definitions

// error: no rules expected the token `!`
four_sides!{ border_ {} _color {
    SpecifiedValue = specified::CSSColor;
    mod private {}
}}
border_top_style {
    SpecifiedValue = self::private::BroderStyle;

    mod private {
        #[allow(non_camel_case_types)]
        #[deriving(Clone)]
        pub enum BroderStyle {
            none,
            solid,
        }
    }
}
border_top_width {
    SpecifiedValue = specified::Length;
    mod private {}
}

border_left_style {
    SpecifiedValue = self::private::BroderStyle;

    mod private {
        #[allow(non_camel_case_types)]
        #[deriving(Clone)]
        pub enum BroderStyle {
            none,
            solid,
        }
    }
}
border_left_width {
    SpecifiedValue = specified::Length;
    mod private {}
}

// End of longhand definitions

}}}
