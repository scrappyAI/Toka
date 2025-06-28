# toka-memory-api

Toka Memory – API crate

This lightweight crate exposes the [`MemoryAdapter`] trait used across the
workspace for ephemeral key–value caching.  It purposefully avoids heavy
dependencies so alternate back-ends (e.g. Redis, sled) can implement the
contract without carrying unnecessary baggage.

License: MIT OR Apache-2.0
