# async_io

This is an asynchronous runtime written in Rust. It fills the same role as the async-std and Tokio crates. Inspiration was taken from Cppcoro and Boost Asio.

When writing server-like programs, it is easier to reason about forked processes and blocking I/O, than polling and threadpooling. Liberally spawning processes (or threads), then blocking on socket I/O calls, incurs large performance penalties.

The purpose of a runtime is to provide the best of both worlds. The user writes conceptually blocking and forking code, and the runtime takes care of executing the code in a non-blocking and thread-pooled fashion.
