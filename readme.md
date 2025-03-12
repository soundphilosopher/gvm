# Golang Version Manager

## Limitations

- Can only be build on Linux systems
- Only supports Bash and Zsh shells

## Pre-requisits

- Rust

## Install

```shell
# build and install GVM
$ cargo install --path .

# init GVM
$ gvm init

# get latest releases
$ gvm update
```

## List available releases

```shell
$ gvm list-remote
$ gvm ls-remote
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
