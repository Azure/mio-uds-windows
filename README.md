# mio-uds-windows

A library for integrating Unix Domain Sockets with [mio] on Windows. Similar to
the standard library's [support for Unix sockets][std], except all of the
abstractions and types are nonblocking to conform with the expectations of mio.

```toml
# Cargo.toml
[dependencies]
mio-uds-windows = "0.1.0"
mio = "0.6"
```

## Structure

The two exported types at the top level, `UnixStream` and `UnixListener`, are
Windows ports of their TCP counterparts in the [mio] library. They can be used
in similar fashion to mio's TCP and UDP types in terms of registration and API.

Most of the exported types in `mio_uds_windows::net` are analagous to the
Unix-specific types in [std], but have been adapted for Windows.

Two "extension" traits, `UnixListenerExt` and `UnixStreamExt`, and their
implementations, were adapted from their TCP counterparts in the [miow] library.

## Windows support for Unix domain sockets
Support for Unix domain sockets was introduced in Windows 10
[Insider Build 17063][af-unix-preview]. It became generally available in version
1809 (aka the October 2018 Update), and in Windows Server 1809/2019.

[af-unix-preview]: https://blogs.msdn.microsoft.com/commandline/2017/12/19/af_unix-comes-to-windows
[mio]: https://github.com/carllerche/mio
[std]: https://doc.rust-lang.org/std/os/unix/net/
[miow]: https://github.com/alexcrichton/miow

# License

This project is licensed under MIT license ([LICENSE-MIT](LICENSE-MIT) or
http://opensource.org/licenses/MIT).

# Contributing

This project welcomes contributions and suggestions.  Most contributions require you to agree to a
Contributor License Agreement (CLA) declaring that you have the right to, and actually do, grant us
the rights to use your contribution. For details, visit https://cla.microsoft.com.

When you submit a pull request, a CLA-bot will automatically determine whether you need to provide
a CLA and decorate the PR appropriately (e.g., label, comment). Simply follow the instructions
provided by the bot. You will only need to do this once across all repos using our CLA.

This project has adopted the [Microsoft Open Source Code of Conduct](https://opensource.microsoft.com/codeofconduct/).
For more information see the [Code of Conduct FAQ](https://opensource.microsoft.com/codeofconduct/faq/) or
contact [opencode@microsoft.com](mailto:opencode@microsoft.com) with any additional questions or comments.
