class RocksdbServer < Formula
  desc "RocksDB server is designed to handle database operations over TCP connections"
  homepage "https://github.com/s00d/RocksDBFusion"
  version "{VERSION}"  # Эта версия будет заменена через GitHub Actions

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/s00d/RocksDBFusion/releases/download/server-v{VERSION}/rocksdb_server-Darwin-x86_64.tar.gz"
      sha256 "{MACOS_X86_64_SHA256}"
    elsif Hardware::CPU.arm?
      url "https://github.com/s00d/RocksDBFusion/releases/download/server-v{VERSION}/rocksdb_server-Darwin-aarch64.tar.gz"
      sha256 "{MACOS_AARCH64_SHA256}"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/s00d/RocksDBFusion/releases/download/server-v{VERSION}/rocksdb_server-Linux-x86_64-musl.tar.gz"
      sha256 "{LINUX_X86_64_SHA256}"
    elsif Hardware::CPU.arm?
      url "https://github.com/s00d/RocksDBFusion/releases/download/server-v{VERSION}/rocksdb_server-Linux-aarch64-musl.tar.gz"
      sha256 "{LINUX_AARCH64_SHA256}"
    end
  end

  def install
    bin.install "rocksdb_server"

    # Make the binary executable
    chmod "+x", "#{bin}/rocksdb_server"

    # Clear extended attributes and sign the binary (macOS only)
    if OS.mac?
      ohai "During the installation process, you will be prompted to enter your password."
      ohai "This is necessary to make the binary executable and to self-sign the application"
      ohai "using the `xattr` and `codesign` commands to ensure it runs correctly on macOS."

      system "/usr/bin/xattr", "-cr", "#{bin}/rocksdb_server"
      system "/usr/bin/codesign", "--force", "--deep", "--sign", "-", "#{bin}/rocksdb_server"
    end
  end

  service do
    run [opt_bin/"rocksdb_server"]
    keep_alive true
    working_dir var
    log_path var/"log/rocksdb_server.log"
    error_log_path var/"log/rocksdb_server.log"
    run_type :immediate
  end

  test do
    system "#{bin}/rocksdb_server", "--version"
  end
end
