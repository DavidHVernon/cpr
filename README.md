# cpr

A fast replacement for cp -R.

## Description

Cpr is a drop in replacement for the unix utility cp with the recursive (-R) option (ie. cp -R). The difference? It's about five to six times faster.

### Example
```
$ cpr ~/big-dir ~/big-dir-copy
```

## Install 
### Linux, Mac, WSL
Run the following from a terminal.
```
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/DavidHVernon/cpr/master/install.sh)"
```
### Windows
```
Coming soon...
```

## Build from Source

If you don't have rust installed: https://www.rust-lang.org/tools/install.
Then...
```
$ cargo build --release
$ ./install.sh
```

## Author

[David Vernon](email:davidhvernon@mac.com)

## Version History

* 0.1.0
    * Initial release.

## License

This project is licensed under the MIT License - see the [license-mit.md](license-mit.md) file for details.

