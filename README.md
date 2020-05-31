# `serial-terminal-rs`

[![Build Status](https://travis-ci.com/mvertescher/serial-terminal-rs.svg?branch=master)](https://travis-ci.com/mvertescher/serial-terminal-rs)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> A very simple interactive serial terminal

```text
serial-terminal 0.0.0
An interactive serial terminal

USAGE:
    serial-terminal [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -l, --list       Lists available serial ports
    -V, --version    Prints version information

OPTIONS:
    -b, --baud <baud>                    Baud rate [default: 921600]
    -d, --data-bits <data-bits>          Data bits (5, 6, 7, 8) [default: 8]
        --eol <eol>                      End of line transformation (cr, lf, crlf) [default: crlf]
        --flow-control <flow-control>    Flow control [default: none]
        --parity <parity>                Parity checking (none/odd/even) [default: none]
        --stop-bits <stop-bits>          Stop bits (1, 2) [default: 1]
    -t, --tty <tty>                      Path to the serial device
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
