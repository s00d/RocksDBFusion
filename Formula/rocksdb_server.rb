class RocksdbServer < Formula
  desc "RocksDB server is designed to handle database operations over TCP connections"
  homepage "https://github.com/s00d/RocksDBFusion"
  version "0.3.4"  # Эта версия будет заменена через GitHub Actions

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/s00d/RocksDBFusion/releases/download/server-v0.3.4/rocksdb_server-Darwin-x86_64.tar.gz"
      sha256 "0b884ac72811021221e9581e03194935e0f697c7de14f6b8da77ee9cf047f76c"
    elsif Hardware::CPU.arm?
      url "https://github.com/s00d/RocksDBFusion/releases/download/server-v0.3.4/rocksdb_server-Darwin-aarch64.tar.gz"
      sha256 "d5eb70db83121bd40edac153785ca4a287cde558dce59f96bed590bc6a6bca0f"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/s00d/RocksDBFusion/releases/download/server-v0.3.4/rocksdb_server-Linux-x86_64-musl.tar.gz"
      sha256 "9d3801d78e72f2a23b21d4deebb558f572a8566ac6b2a206e47fd381fb7de5e1"
    elsif Hardware::CPU.arm?
      url "https://github.com/s00d/RocksDBFusion/releases/download/server-v0.3.4/rocksdb_server-Linux-aarch64-musl.tar.gz"
      sha256 "f3575e0d6ccd51669c882c8e72c2380cef609c29c0b7de349cf0573c72b037ea"
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
