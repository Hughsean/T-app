#![allow(dead_code)]
pub type AsyncRwLock<T> = tokio::sync::RwLock<T>;
pub type AsyncMutex<T> = tokio::sync::Mutex<T>;
pub type SharedAsyncRwLock<T> = std::sync::Arc<AsyncRwLock<T>>;
pub type SharedAsyncMutex<T> = std::sync::Arc<AsyncMutex<T>>;
pub type SharedRwLock<T> = std::sync::Arc<std::sync::RwLock<T>>;
pub type SharedMutex<T> = std::sync::Arc<std::sync::Mutex<T>>;

// pub type AsyncRwLock<T>(tokio::sync::RwLock<T>);
// pub type AsyncMutex<T>(tokio::sync::Mutex<T>);
// pub type SharedAsyncRwLock<T>(std::sync::Arc<AsyncRwLock<T>>);
// pub type SharedAsyncMutex<T>(std::sync::Arc<AsyncMutex<T>>);
// pub type SharedRwLock<T>(std::sync::Arc<std::sync::RwLock<T>>);
// pub type SharedMutex<T>(std::sync::Arc<std::sync::Mutex<T>>);

// impl<T> AsyncRwLock<T> {
//     pub fn new(data: T) -> Self {
//         Self(tokio::sync::RwLock::new(data))
//     }
//     pub async fn read(&self) -> tokio::sync::RwLockReadGuard<'_, T> {
//         self.0.read().await
//     }
//     pub async fn write(&self) -> tokio::sync::RwLockWriteGuard<'_, T> {
//         self.0.write().await
//     }
//     pub fn blocking_read(&self) -> tokio::sync::RwLockReadGuard<'_, T> {
//         self.0.blocking_read()
//     }
//     pub fn blocking_write(&self) -> tokio::sync::RwLockWriteGuard<'_, T> {
//         self.0.blocking_write()
//     }
// }
// impl<T> AsyncMutex<T> {
//     pub fn new(data: T) -> Self {
//         Self(tokio::sync::Mutex::new(data))
//     }
//     pub async fn lock(&self) -> tokio::sync::MutexGuard<'_, T> {
//         self.0.lock().await
//     }
//     pub fn blocking_lock(&self) -> tokio::sync::MutexGuard<'_, T> {
//         self.0.blocking_lock()
//     }
// }
// impl<T> SharedAsyncRwLock<T> {
//     pub fn new(data: T) -> Self {
//         Self(std::sync::Arc::new(AsyncRwLock::new(data)))
//     }
//     pub async fn read(&self) -> tokio::sync::RwLockReadGuard<'_, T> {
//         self.0.read().await
//     }
//     pub async fn write(&self) -> tokio::sync::RwLockWriteGuard<'_, T> {
//         self.0.write().await
//     }
//     pub fn blocking_read(&self) -> tokio::sync::RwLockReadGuard<'_, T> {
//         self.0.blocking_read()
//     }
//     pub fn blocking_write(&self) -> tokio::sync::RwLockWriteGuard<'_, T> {
//         self.0.blocking_write()
//     }
//     pub fn clone(&self) -> Self {
//         Self(self.0.clone())
//     }
// }
// impl<T> SharedAsyncMutex<T> {
//     pub fn new(data: T) -> Self {
//         Self(std::sync::Arc::new(AsyncMutex::new(data)))
//     }
//     pub async fn lock(&self) -> tokio::sync::MutexGuard<'_, T> {
//         self.0.lock().await
//     }
//     pub fn blocking_lock(&self) -> tokio::sync::MutexGuard<'_, T> {
//         self.0.blocking_lock()
//     }
//     pub fn clone(&self) -> Self {
//         Self(self.0.clone())
//     }
// }
// impl<T> SharedRwLock<T> {
//     pub fn new(data: T) -> Self {
//         Self(std::sync::Arc::new(std::sync::RwLock::new(data)))
//     }
//     pub fn read(&self) -> std::sync::LockResult<std::sync::RwLockReadGuard<'_, T>> {
//         self.0.read()
//     }
//     pub fn write(&self) -> std::sync::LockResult<std::sync::RwLockWriteGuard<'_, T>> {
//         self.0.write()
//     }
//     pub fn clone(&self) -> Self {
//         Self(self.0.clone())
//     }
// }

// impl<T> SharedMutex<T> {
//     pub fn new(data: T) -> Self {
//         Self(std::sync::Arc::new(std::sync::Mutex::new(data)))
//     }
//     pub fn lock(&self) -> std::sync::LockResult<std::sync::MutexGuard<'_, T>> {
//         self.0.lock()
//     }
//     pub fn clone(&self) -> Self {
//         Self(self.0.clone())
//     }
// }
