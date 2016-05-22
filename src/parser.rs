// pest. Smart PEGs in Rust
// Copyright (C) 2016  Dragoș Tiselice
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::VecDeque;

/// A `trait` that defines a parser.
pub trait Parser {
    type Rules;

    /// Matches `string`, returns whether it matched, and advances a parser with `string.len()` in
    /// case it did.
    fn matches(&mut self, string: &str) -> bool;

    /// Matches `char` between `left` and `right`, and advances a parser with one `char` in case
    /// it did.
    fn between(&mut self, left: char, right: char) -> bool;

    /// Tries to match `rule`, returns whether it matched, and advances a parser with in case it
    /// did. If `revert` is `true`, the parser will not advance.
    fn try<F>(&mut self, revert: bool, rule: F) -> bool where F: FnOnce(&mut Self) -> bool;

    /// Returns the current position of a `Parser`.
    fn pos(&self) -> usize;

    /// Sets the position of a `Parser`.
    fn set_pos(&mut self, pos: usize);

    /// Returns whether a `Parser` has reached it end.
    fn end(&self) -> bool;

    /// Reset a `Parser`.
    fn reset(&mut self);

    /// Returns the queue of all matched `Rules`.
    fn queue(&mut self) -> &mut VecDeque<Self::Rules>;

    /// Skips white-space.
    fn skip_ws(&mut self);
}
