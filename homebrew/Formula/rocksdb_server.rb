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
      (bin/"rocksdb_server_wrapper").write <<~EOS
        #!/bin/bash
        export ROCKSDB_PATH=${ROCKSDB_PATH:-#{var}/rocksdb/db}
        export ROCKSDB_PORT=${ROCKSDB_PORT:-12345}
        export ROCKSDB_LOCK_FILE=${ROCKSDB_LOCK_FILE:-#{var}/rocksdb/rocksdb.lock}
        exec #{opt_bin}/rocksdb_server --dbpath $ROCKSDB_PATH --port $ROCKSDB_PORT --lock-file $ROCKSDB_LOCK_FILE --host 127.0.0.1 --log-level info
      EOS
      chmod 0755, bin/"rocksdb_server_wrapper"
    end

    service do
      run [opt_bin/"rocksdb_server_wrapper"]
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
