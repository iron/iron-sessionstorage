//! You can choose between multiple backends to store your session data. The easiest to manage is
//! `SignedCookieBackend`. You need to compile with the `redis-backend` feature to use the Redis
//! backend.

mod signedcookie;
pub use self::signedcookie::SignedCookieBackend;

#[cfg(feature = "redis-backend")]
mod redis;
#[cfg(feature = "redis-backend")]
pub use self::redis::RedisBackend;

