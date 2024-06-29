cask "rocksdb-viewer" do
  version "{VERSION}"
  sha256 "{MACOS_X86_64_SHA256}", macos: :monterey
  sha256 "{MACOS_AARCH64_SHA256}", macos: :arm64

  url "https://github.com/s00d/RocksDBFusion/releases/download/app-v{VERSION}/rocksdb-viewer_{VERSION}_x64.dmg", if Hardware::CPU.intel?
  url "https://github.com/s00d/RocksDBFusion/releases/download/app-v{VERSION}/rocksdb-viewer_{VERSION}_aarch64.dmg", if Hardware::CPU.arm?

  name "RocksDB Viewer"
  desc "A simple Tauri application to view and interact with a RocksDB database"
  homepage "https://github.com/s00d/RocksDBFusion"

  app "rocksdb-viewer.app"

  zap trash: [
    "~/Library/Preferences/com.rocksdb.viewer.plist",
    "~/Library/Saved Application State/com.rocksdb.viewer.savedState",
  ]
end
