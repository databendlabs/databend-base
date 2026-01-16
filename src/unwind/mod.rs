/// Panic-safe wrapper for code that might panic during Drop.
///
/// When code panics during Drop while already unwinding from another panic
/// (double panic), Rust aborts immediately. This function captures a backtrace
/// before that happens, providing visibility into the double-panic scenario.
///
/// # Usage
///
/// ```ignore
/// impl Drop for MyType {
///     fn drop(&mut self) {
///         drop_guard(|| {
///             // cleanup code that might panic
///             self.resource.close();
///         });
///     }
/// }
/// ```
pub fn drop_guard<F: FnOnce() -> R, R>(f: F) -> R {
    let panicking = std::thread::panicking();
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(res) => res,
        Err(panic) => {
            if panicking {
                eprintln!("double panic");

                let backtrace = std::backtrace::Backtrace::force_capture();
                eprintln!("double panic {:?}", backtrace);
                log::error!("double panic {:?}", backtrace);
            }

            std::panic::resume_unwind(panic)
        }
    }
}
