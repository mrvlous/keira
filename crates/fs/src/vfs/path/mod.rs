// SPDX-License-Identifier: GPL-2.0-only

//! Path normalization, standard alias resolution and filesystem target routing.

pub mod alias;
pub mod router;

pub use self::alias::resolve_alias_path;
pub use self::router::route_path;
