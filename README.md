# iron-sessionstorage

Flask-inspired session middleware with multiple possible backends.

Like in Flask, a signed cookie backend is implemented by default. The
implementation is very similar to [oven](https://github.com/flosse/oven), with
the exception that you could write a new session backend that uses serverside
session storage instead of signed cookies.

See examples for usage.

## License

Licensed under the MIT, see `LICENSE`.
