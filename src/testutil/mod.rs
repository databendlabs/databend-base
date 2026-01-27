//! Utilities for local development and testing.
//!
//! Provides helpers for acquiring local resources like network ports.

use std::net::TcpListener;

/// Get a [`TcpListener`] bound to an available port.
///
/// Returns the listener directly so the port stays reserved until the caller
/// is done with it. Use `listener.local_addr().unwrap().port()` to get the port.
///
/// This avoids the TOCTOU race in [`next_port()`] where the OS might recycle
/// the port between release and actual use.
pub fn next_listener() -> TcpListener {
    TcpListener::bind("127.0.0.1:0").expect("Failed to bind to port 0")
}

/// Get an available port by asking the OS to assign one.
///
/// This binds to port 0, retrieves the assigned port, then drops the listener.
/// There's a small race window between dropping and actual use, but it's
/// acceptable for tests and avoids conflicts between parallel test processes.
///
/// For reliable port reservation, use [`next_listener()`] instead.
pub fn next_port() -> u16 {
    next_listener().local_addr().expect("Failed to get local address").port()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::net::TcpListener;
    use std::sync::Mutex;
    use std::thread;

    use super::*;

    #[test]
    fn test_next_listener() {
        let listener = next_listener();
        let port = listener.local_addr().unwrap().port();
        assert!(port > 0, "Port should be positive: {}", port);

        let listener2 = next_listener();
        let port2 = listener2.local_addr().unwrap().port();
        assert_ne!(port, port2, "Listeners should have different ports");
    }

    #[test]
    fn test_next_listener_concurrent() {
        const NUM_THREADS: usize = 4;
        const LISTENERS_PER_THREAD: usize = 50;
        const TOTAL: usize = NUM_THREADS * LISTENERS_PER_THREAD;

        let listeners: Mutex<Vec<TcpListener>> = Mutex::new(Vec::with_capacity(TOTAL));

        thread::scope(|s| {
            for _ in 0..NUM_THREADS {
                s.spawn(|| {
                    for _ in 0..LISTENERS_PER_THREAD {
                        let listener = next_listener();
                        listeners.lock().unwrap().push(listener);
                    }
                });
            }
        });

        let guard = listeners.lock().unwrap();
        let ports: HashSet<u16> = guard.iter().map(|l| l.local_addr().unwrap().port()).collect();
        assert_eq!(
            ports.len(),
            TOTAL,
            "Expected {} unique ports, got {} (duplicates detected)",
            TOTAL,
            ports.len()
        );
    }

    #[test]
    fn test_next_port() {
        let port = next_port();
        assert!(port > 0, "Port should be positive: {}", port);

        // Ports should be unique
        let port2 = next_port();
        assert_ne!(port, port2, "Consecutive ports should differ");
    }

    /// The OS may recycle a released port before it's actually used,
    /// causing duplicate assignments under concurrent load.
    #[test]
    #[ignore]
    fn test_next_port_concurrent() {
        const NUM_THREADS: usize = 4;
        const PORTS_PER_THREAD: usize = 50;
        const TOTAL_PORTS: usize = NUM_THREADS * PORTS_PER_THREAD;

        let ports: Mutex<HashSet<u16>> = Mutex::new(HashSet::with_capacity(TOTAL_PORTS));

        thread::scope(|s| {
            for _ in 0..NUM_THREADS {
                s.spawn(|| {
                    for _ in 0..PORTS_PER_THREAD {
                        let port = next_port();
                        let mut guard = ports.lock().unwrap();
                        guard.insert(port);
                    }
                });
            }
        });

        let unique_count = ports.lock().unwrap().len();
        assert_eq!(
            unique_count, TOTAL_PORTS,
            "Expected {} unique ports, got {} (duplicates detected)",
            TOTAL_PORTS, unique_count
        );
    }
}
