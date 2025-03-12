# Golang Version Manager

## Limitations

- Can only be build on Linux systems
- Only supports Bash and Zsh shells

## Pre-requisits

- Rust

## Install

```shell
$ ./install-nix.sh
```

## List available releases

```shell
# all versions
$ gvm list-remote
$ gvm ls-remote

# only stable versions
$ gvm list-remote --stable
$ gvm ls-remote --stable

# list with specific version
$ gvm list-remote 1.24.0
$ gvm ls-remote 1.24.0

# list with wildcard version
$ gvm list-remote 1.24.*
$ gvm ls-remote 1.24.*
```

## List installed versions

```shell
# all versions
$ gvm list
$ gvm ls

# only stable versions
$ gvm list --stable
$ gvm ls --stable

# list with specific version
$ gvm list 1.24.0
$ gvm ls 1.24.0

# list with wildcard version
$ gvm list 1.24.*
$ gvm ls 1.24.*
```

## Install Go

```shell
# without activation
$ gvm install 1.24.1

# activate version
$ gvm install 1.24.1 --use
```

## Load Go

```shell
# optonal if not already done with --use in install
$ gvm use 1.24.1

# reload profile
$ source .profile

# test
$ go version
```

## Help

```shell
$ go help
```
