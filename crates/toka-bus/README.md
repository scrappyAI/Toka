# toka-bus

## Toka Bus

Process-local, in-memory event propagation layer.
Other crates (agents, runtime, vault) interact only through this
abstraction rather than talking to each other directly.  In turn this
keeps the system loosely coupled and far easier for humans – and LLMs –
to comprehend.

License: MIT OR Apache-2.0
