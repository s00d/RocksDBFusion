class RocksdbCli < Formula
  desc "RocksDB CLI client for database operations"
  homepage "https://github.com/s00d/RocksDBFusion"
  version "0.1.1"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/s00d/RocksDBFusion/releases/download/rocksdb-cli-v0.1.1/rocksdb_cli-Darwin-x86_64.tar.gz"
      sha256 "b976f311631842fd5c02fca48b47bf6a05a2accf799a9883a03027b781192f5c"
    elsif Hardware::CPU.arm?
      url "https://github.com/s00d/RocksDBFusion/releases/download/rocksdb-cli-v0.1.1/rocksdb_cli-Darwin-aarch64.tar.gz"
      sha256 "2b74a1a14d25dd08ead931a201c6444e261a1ad9259100c03931b0a412aa65f8"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/s00d/RocksDBFusion/releases/download/rocksdb-cli-v0.1.1/rocksdb_cli-Linux-x86_64-musl.tar.gz"
      sha256 "d337212aa254ef837efee9dc85109ac54ca1ddd75e5c782558c31189c69bfb91"
    elsif Hardware::CPU.arm?
      url "https://github.com/s00d/RocksDBFusion/releases/download/rocksdb-cli-v0.1.1/rocksdb_cli-Linux-aarch64-musl.tar.gz"
      sha256 "67bbb9b9ec8a3c8b5727216fdaa2864e13f8bf5032a6018cc95e44165e745f80"
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
