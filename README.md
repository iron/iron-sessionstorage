# iron-sessionstorage [![Build Status](https://travis-ci.org/iron/iron-sessionstorage.svg?branch=master)](https://travis-ci.org/iron/iron-sessionstorage)

- [Documentation](https://docs.rs/iron-sessionstorage)
- [Repository](https://github.com/iron/iron-sessionstorage)
- [Crates.io](https://crates.io/crates/iron-sessionstorage)

Session middleware for Iron, allows you to store data in a simple
type-to-string map for each user.

See examples for usage.

## Backends

You can use one of the included backends for data storage or roll your own:

- A cookie-based backend is available by default. You will need to provide a
  key with which values will be signed.

- A redis backend can be enabled using the `redis-backend` feature.

## License

Licensed under the MIT, see `LICENSE`.
