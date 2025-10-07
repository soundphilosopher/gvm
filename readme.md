# 🚀 GVM - Go Version Manager

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=for-the-badge)](./LICENSE)
<!--[![Build Status](https://img.shields.io/github/workflow/status/yourusername/gvm/CI?style=for-the-badge)](https://github.com/yourusername/gvm/actions)
[![Version](https://img.shields.io/badge/version-2025.3.12-blue.svg?style=for-the-badge)](https://github.com/yourusername/gvm/releases)-->

> *Because managing Go versions shouldn't be harder than writing Go code itself!* 🎯

**GVM** is a blazingly fast Go Version Manager built in Rust, designed specifically for Linux systems. It lets you effortlessly install, manage, and switch between different Go versions with the grace of a gopher and the speed of a rocket! 🐹⚡

## ✨ Features

- 🏃‍♂️ **Lightning Fast** - Built in Rust for maximum performance
- 🔄 **Easy Switching** - Change Go versions in seconds
- 📦 **Simple Installation** - One-command setup
- 🎯 **Version Filtering** - Find exactly the version you need
- 🔗 **Smart Aliasing** - Create memorable shortcuts for your favorite versions
- 🌟 **Shell Integration** - Works seamlessly with Bash and Zsh
- 💾 **Caching** - Smart caching for faster subsequent operations

## 🎪 Quick Demo

```bash
# Install and activate the latest stable Go version
gvm install 1.21.5 --use

# Switch between versions like a pro
gvm use 1.20.10

# Create aliases for your favorite versions
gvm alias stable 1.21.5
gvm alias experimental 1.22rc1

# List what's available in the Go universe
gvm list-remote --stable
```

## 🚀 Installation

### Prerequisites

- **Linux** (Sorry Windows and macOS folks, we're Linux-exclusive! 🐧)
- **Rust** (The language of systems programming gods)
- **Bash or Zsh** shell

### The Magic One-Liner

```bash
git clone https://github.com/yourusername/gvm.git
cd gvm
./install-nix.sh
```

This script will:

1. 🔧 Build and install GVM using Cargo
2. 🎯 Initialize the Go environment
3. 📡 Update the version cache
4. 🏆 Install the latest stable Go version
5. 🔄 Reload your profile

### Manual Installation

If you prefer the scenic route:

```bash
# Clone the repository
git clone https://github.com/yourusername/gvm.git
cd gvm

# Build and install
cargo install --path .

# Initialize GVM
gvm init
gvm update

# Install your first Go version
gvm install 1.21.5 --use

# Reload your shell
source ~/.profile
```

## 📚 Usage Guide

### 🔍 Discovering Go Versions

```bash
# List all available versions
gvm list-remote
gvm ls-remote  # Short alias because typing is hard

# Only show stable releases (recommended for production)
gvm list-remote --stable

# Find a specific version
gvm list-remote 1.21.0

# Wildcard search (find all 1.21.x versions)
gvm list-remote 1.21.*
```

### 📦 Installing Go Versions

```bash
# Install a specific version
gvm install 1.21.5

# Install and immediately activate
gvm install 1.21.5 --use

# The --use flag is your friend for quick setups!
```

### 🔄 Managing Installed Versions

```bash
# See what you've got installed
gvm list
gvm ls  # Because brevity is the soul of wit

# Filter your installed versions
gvm list --stable
gvm list 1.21.*

# Switch to a different version
gvm use 1.20.10

# Remove versions you no longer need
gvm remove 1.19.13
```

### 🏷️ Smart Aliasing

```bash
# Create meaningful aliases
gvm alias production 1.21.5
gvm alias development 1.22rc1
gvm alias legacy 1.19.13

# Use your aliases
gvm use production

# List all your aliases
gvm alias

# Remove aliases when they're no longer needed
gvm remove-alias legacy
```

### 🛠️ Shell Integration

```bash
# Get shell completions (supports bash, zsh, fish, etc.)
gvm completions bash >> ~/.bashrc
gvm completions zsh >> ~/.zshrc

# Update your version cache
gvm update

# Get help when you're stuck
gvm help
```

## 🎯 Pro Tips

1. **Always use `--stable`** for production environments
2. **Create aliases** for versions you use frequently
3. **Run `gvm update`** regularly to get the latest Go releases
4. **Use wildcards** to quickly find version families
5. **The `--use` flag** saves you a step when installing

## 🔧 Configuration

GVM stores everything in your home directory:

- **Versions**: `~/.gvm/versions/`
- **Aliases**: `~/.gvm/aliases/`
- **Cache**: `~/.gvm/cache/`

## 🐛 Known Limitations

- 🐧 **Linux Only** - We're platform-specific by design
- 🐚 **Bash/Zsh Only** - Fish lovers, we hear you (contributions welcome!)
- 🔄 **Profile Reload Required** - You might need to `source ~/.profile` after switching versions

## 🤝 Contributing

Found a bug? Have a cool feature idea? We'd love your help!

1. Fork this repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under MIT - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- The Go team for creating an amazing language
- The Rust community for the incredible tooling
- All the gophers out there writing awesome Go code

---

<div align="center">

**Made with ❤️ and lots of ☕ by developers, for developers**

[Report Bug](https://github.com/yourusername/gvm/issues) • [Request Feature](https://github.com/yourusername/gvm/issues) • [Contribute](https://github.com/yourusername/gvm/pulls)

</div>
