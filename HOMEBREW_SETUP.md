# Homebrew Setup Guide

This guide explains how to set up and maintain a Homebrew tap for distributing codesandbox.

## What You'll Need

1. A GitHub repository for your Homebrew tap (e.g., `homebrew-codesandbox`)
2. Tagged releases of your project
3. SHA256 checksums for your release archives

## Setting Up Your Homebrew Tap

### Step 1: Create a Homebrew Tap Repository

Create a new GitHub repository with the naming convention `homebrew-<name>`. For example:
- `homebrew-codesandbox`

### Step 2: Add the Formula to Your Tap

1. Copy the `codesandbox.rb` file from this repository to your tap repository
2. Place it in the `Formula/` directory (create if it doesn't exist)
3. Update the formula with the correct details:

```ruby
class Codesandbox < Formula
  desc "Create isolated Ubuntu Docker containers with Claude Code pre-installed"
  homepage "https://github.com/your-username/codesandbox"
  url "https://github.com/your-username/codesandbox/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "YOUR_SHA256_HERE"  # Generate this from the release archive
  license "MIT"  # Update based on your actual license

  depends_on "rust" => :build
  depends_on "docker"

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    # Test that the binary was installed and can show help
    assert_match "Create isolated Ubuntu Docker containers", shell_output("#{bin}/codesandbox --help")
    
    # Test version output
    assert_match version.to_s, shell_output("#{bin}/codesandbox --version")
  end
end
```

### Step 3: Generate SHA256 Checksum

When you create a new release, calculate the SHA256 checksum:

```bash
# Download the release archive
curl -L -o codesandbox-v0.1.0.tar.gz https://github.com/your-username/codesandbox/archive/refs/tags/v0.1.0.tar.gz

# Generate SHA256
shasum -a 256 codesandbox-v0.1.0.tar.gz
```

Update the `sha256` field in your formula with this value.

### Step 4: Test Your Formula Locally

Before publishing, test the formula locally:

```bash
# Install from your local tap
brew install --build-from-source ./Formula/codesandbox.rb

# Run the tests
brew test codesandbox

# Audit the formula
brew audit --strict --online codesandbox
```

## Using Your Homebrew Tap

Once your tap is set up, users can install codesandbox like this:

```bash
# Add your tap
brew tap your-username/codesandbox

# Install codesandbox
brew install codesandbox
```

Or in one command:

```bash
brew install your-username/codesandbox/codesandbox
```

## Updating the Formula

When you release a new version:

1. Create a new tagged release on GitHub
2. Calculate the new SHA256 checksum
3. Update the `url`, `sha256`, and version in your formula
4. Commit and push the changes to your tap repository

## Best Practices

1. **Use Semantic Versioning**: Follow semantic versioning for your releases
2. **Test Thoroughly**: Always test new formula versions before publishing
3. **Keep Dependencies Minimal**: Only add necessary dependencies
4. **Write Good Tests**: Ensure your test block validates core functionality
5. **Follow Homebrew Style**: Use `brew audit` to ensure your formula follows conventions

## Troubleshooting

### Common Issues

1. **SHA256 Mismatch**: Make sure you're calculating the checksum for the correct file
2. **Build Failures**: Ensure all Rust dependencies are properly specified
3. **Test Failures**: Make sure your binary supports the expected command-line flags

### Getting Help

- Check the [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- Browse existing formulas for examples
- Use `brew audit` to catch common issues

## Example Tap Structure

```
homebrew-codesandbox/
├── Formula/
│   └── codesandbox.rb
└── README.md
```

This is the minimal structure needed for a Homebrew tap.