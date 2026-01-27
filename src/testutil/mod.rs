//! Utilities for local development and testing.
//!
//! Provides helpers for acquiring local resources like network ports.

use std::net::TcpListener;

/// Get an available port by asking the OS to assign one.
///
/// This binds to port 0, retrieves the assigned port, then drops the listener.
/// There's a small race window between dropping and actual use, but it's
/// acceptable for tests and avoids conflicts between parallel test processes.
pub fn next_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to port 0");
    listener.local_addr().expect("Failed to get local address").port()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::Mutex;
    use std::thread;

    use super::*;

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
