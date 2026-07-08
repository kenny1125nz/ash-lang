use std::sync::{Mutex, MutexGuard, OnceLock};

pub fn lock_guard<T>(mu: &Mutex<T>) -> MutexGuard<'_, T> {
    mu.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn get_pool() -> &'static threadpool::ThreadPool {
    static POOL: OnceLock<threadpool::ThreadPool> = OnceLock::new();
    POOL.get_or_init(|| threadpool::ThreadPool::new(4))
}
