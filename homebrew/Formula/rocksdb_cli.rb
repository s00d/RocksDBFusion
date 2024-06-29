class RocksdbCli < Formula
  desc "RocksDB CLI client for database operations"
  homepage "https://github.com/s00d/RocksDBFusion"
  version "{VERSION}"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/s00d/RocksDBFusion/releases/download/rocksdb-cli-v{VERSION}/rocksdb_cli-Darwin-x86_64.tar.gz"
      sha256 "{MACOS_X86_64_SHA256}"
    elsif Hardware::CPU.arm?
      url "https://github.com/s00d/RocksDBFusion/releases/download/rocksdb-cli-v{VERSION}/rocksdb_cli-Darwin-aarch64.tar.gz"
      sha256 "{MACOS_AARCH64_SHA256}"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/s00d/RocksDBFusion/releases/download/rocksdb-cli-v{VERSION}/rocksdb_cli-Linux-x86_64-musl.tar.gz"
      sha256 "{LINUX_X86_64_SHA256}"
    elsif Hardware::CPU.arm?
      url "https://github.com/s00d/RocksDBFusion/releases/download/rocksdb-cli-v{VERSION}/rocksdb_cli-Linux-aarch64-musl.tar.gz"
      sha256 "{LINUX_AARCH64_SHA256}"
    end
  end

  def install
    bin.install "rocksdb_cli"

    # Make the binary executable
    chmod "+x", "#{bin}/rocksdb_cli"

    # Clear extended attributes and sign the binary (macOS only)
    if OS.mac?
      ohai "During the installation process, you will be prompted to enter your password."
      ohai "This is necessary to make the binary executable and to self-sign the application"
      ohai "using the `xattr` and `codesign` commands to ensure it runs correctly on macOS."

      system "/usr/bin/xattr", "-cr", "#{bin}/rocksdb_cli"
      system "/usr/bin/codesign", "--force", "--deep", "--sign", "-", "#{bin}/rocksdb_cli"
    end
  end

  test do
    system "#{bin}/rocksdb_cli", "--version"
  end

end
