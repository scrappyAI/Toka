# toka-events-api

Event API crate – pure data types and trait contracts for the Toka event
subsystem.  This crate purposefully contains **no storage code** or heavy
dependencies so it can be used in any context, including `no_std` (once the
default features are disabled).

Downstream crates are expected to implement the [`EventSink`] and
[`QueryApi`] traits and provide their own storage back-ends.

License: MIT OR Apache-2.0
