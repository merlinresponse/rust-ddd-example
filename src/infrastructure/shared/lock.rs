use std::sync::{Mutex, MutexGuard};

/// Locks a std `Mutex`, recovering it if a panicking thread "poisoned" it.
/// The data behind our locks is still consistent after a panic, so recovering is safe here.
pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
