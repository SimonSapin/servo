/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![macro_escape]

macro_rules! with_shorthand_definitions { ($macro: ident) => { $macro! {

// Start of shorthand definitions

border_top |_value, _base_url| -> (border_top_color, border_top_style, border_top_width) {
    Ok(Longhands {
        border_top_color: None,
        border_top_style: None,
        border_top_width: Some(specified::Au_(Au(0))),
    })
}

border_left |_value, _base_url| -> (border_left_color, border_left_style, border_left_width) {
    Ok(Longhands {
        border_left_color: None,
        border_left_style: None,
        border_left_width: Some(specified::Au_(Au(0))),
    })
}

// End of shorthand definitions

}}}
