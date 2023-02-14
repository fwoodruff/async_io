# async_io

This is an asynchronous runtime written in Rust. Tokio and Boost Asio are enterprise projects addressing the same problem.

When writing I/O bound multitasking programs, like servers, it is easier to reason about forked processes and blocking I/O, than polling and thread pool management. However liberally spawning processes (or threads), then blocking on socket I/O calls, puts substantial strain on the operating system.

The purpose of a runtime is to provide the best of both worlds. The user writes conceptually blocking and liberally forked code, and the runtime manages execution of the code in a non-blocking and thread-pooled fashion.

As an exposition, in the main.rs file, I am running a simple HTTP server on this runtime at http://freddiewoodruff.co.uk:8080

The public interface is as follows:
```rust
// TCP
fn connect(addr: &SocketAddr) -> Result<TcpStream, Error>
fn Listener::bind(address : &str) -> Result<Self>
async fn Listener::accept(&mut self) -> Result<Stream, Error>
async fn Stream::read(&'a self, buffer : &'a mut [u8] ) -> Result<usize, Error>
async fn Stream::write(&'a self, buffer : &'a [u8]) -> Result<usize, Error>
async fn Stream::write_all(&'a self, buffer : &'a [u8]) -> Result<(), Error>

// Unbounded channels
fn AsyncReceiver::try_receive<T>(&self) -> Result<T, TryRecvError>
async fn AsyncReceiver::receive<T>(&self) -> Result<T, RecvError>
async fn AsyncSender::send<T>(&self, value : T) -> Result<(), SendError<T>>
fn async_channel<T>() -> (AsyncSender<T>, AsyncReceiver<T>)

// Locking
fn AsyncMutex::new<T>(value : T) -> Self
pub async fn lock(&'a self) -> AsyncGuard<'a, T>
AsyncGuard::deref<T>(&self) -> &T
impl<T> Deref for AsyncGuard<'_, T>
impl<T> DerefMut for AsyncGuard<'_, T>

// Ring buffer
fn RingBuffer::new<T>(bound : usize) -> Self 
async fn RingBuffer::send(&self, value : T)
async fn RingBuffer::recv(&self) -> T

// Execution
fn runtime(main_task: impl Future<Output = ()> + Send + 'static)
fn async_spawn(f: impl Future<Output = ()> + Send + 'static) -> JoinFuture
impl Future for JoinFuture
impl Drop for JoinFuture // detach

// sage - main.rs
async fn async_main() {
    // async user code
}
fn main() {
    execution::runtime(async_main());
}
```