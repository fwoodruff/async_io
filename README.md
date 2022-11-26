# async_io

This is an asynchronous runtime written in Rust. Tokio, Goroutines and Boost Asio are enterprise projects addressing the same problem.

When writing I/O bound multitasking programs, like servers, it is easier to reason about forked processes and blocking I/O, than polling and thread pool management. However liberally spawning processes (or threads), then blocking on socket I/O calls, puts substantial strain on the operating system.

The purpose of a runtime is to provide the best of both worlds, an 'OS-lite'. The user writes conceptually blocking and liberally forked code, and the runtime takes care of executing the code in a non-blocking and thread-pooled fashion.

An an exposition, in the main.rs file, I am running a simple HTTP server on this runtime at http://freddiewoodruff.co.uk:8080
