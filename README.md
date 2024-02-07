# Impostor

Impostor is a mock server that uses a human readable, plain text format for defining
mocks.

## Example

Here's how you might define a mock for a simple API:

```
GET /hello
HTTP 200
Content-Type: text/plain
`hello, world!`
```

For a tour of Impostor that goes into more detail, see
[the tour of Impostor.](./samples/tour.impostor)

## Installation

There are currently two ways to use Impostor:

- Download a release from the [releases page](https://github.com/abismoe/impostor/releases)
  and place it in your `PATH`.
- Install from cargo:

    ```sh
    cargo install impostor_cli
    ```

Either way, you should be able to run impostor with `impostor --help`.

## Goals and Philosophy

Impostor aims to be:

- Simple: Impostor should be intuitive and straightforward, and complex features
  that make it harder to reason about how a mock works should be avoided. Like
  Hurl, Impostor aims to keep its format close to HTTP.
- Easy: Impostor's user-facing API should be easy and delightful to use, and new
  users should be able to get started quickly.
- Fast: Impostor should compile and respond to requests as fast as possible.
  There should be as few allocations as possible in the request handling path.

The main goal of Impostor to make authoring mocks a much better experience, and
to make it easy to share those mocks or use them in CI.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for more
information.

## Credits and License

Impostor's text format is adapted from [Hurl](https://hurl.dev), and the parser code
is derived from
[Hurl's.](https://github.com/Orange-OpenSource/hurl/tree/master/packages/hurl_core)
While we are adapting Hurl's format, we do not make any claims of compatibility
and all issues with Impostor or its format should be reported to Impostor, not Hurl.
Please don't bother the Hurl maintainers with issues related to Impostor!

Impostor is licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE)
for the full license text.
