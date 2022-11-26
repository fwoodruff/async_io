# async_io

This is an asynchronous runtime written in Rust. It fills the same role as the async-std and Tokio crates. I took inspiration from Cppcoro and Boost Asio.

When writing server-like programs, it is easier to reason about forked processes and blocking I/O, than polling and thread pool management. However liberally spawning processes (or threads), then blocking on socket I/O calls, incurs large performance and memory penalties.

The purpose of a runtime is to provide the best of both worlds. The user writes conceptually blocking and liberally forked code, and the runtime takes care of executing the code in a non-blocking and thread-pooled fashion.

An an exposition, I am running a simple HTTP server on this runtime at http://freddiewoodruff.co.uk:8080
