# axum-xml

[![crates.io](https://img.shields.io/crates/v/axum-xml?style=flat-square)](https://crates.io/crates/axum-xml)
[![Documentation](https://img.shields.io/docsrs/axum-xml?style=flat-square)](https://docs.rs/axum-xml)

XML extractor for axum.

This crate provides struct `Xml` that can be used to extract typed information from request's body.

Under the hood, [quick-xml](https://github.com/tafia/quick-xml) is used to parse payloads.

## Features

- `encoding`: support non utf-8 payload

## License

MIT