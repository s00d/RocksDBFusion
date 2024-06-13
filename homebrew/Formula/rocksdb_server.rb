class RocksdbFusion < Formula
  desc "Description of RocksDBFusion"
  homepage "https://github.com/s00d/RocksDBFusion"
  version "#{version}"  # Эта версия будет заменена через GitHub Actions

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/s00d/RocksDBFusion/releases/download/server-v#{version}/rocksdb_server-Darwin-x86_64.tar.gz"
      sha256 "your_sha256_checksum_for_x86_64"
    elsif Hardware::CPU.arm?
      url "https://github.com/s00d/RocksDBFusion/releases/download/server-v#{version}/rocksdb_server-Darwin-aarch64.tar.gz"
      sha256 "your_sha256_checksum_for_aarch64"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/s00d/RocksDBFusion/releases/download/server-v#{version}/rocksdb_server-Linux-x86_64-musl.tar.gz"
      sha256 "your_sha256_checksum_for_linux_x86_64"
    elsif Hardware::CPU.arm?
      url "https://github.com/s00d/RocksDBFusion/releases/download/server-v#{version}/rocksdb_server-Linux-aarch64-musl.tar.gz"
      sha256 "your_sha256_checksum_for_linux_aarch64"
    end
  end

  def install
    bin.install "rocksdb_server"
  end

  test do
    system "#{bin}/rocksdb_server", "--version"
  end
end
