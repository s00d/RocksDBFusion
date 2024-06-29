cask "rocksdb-viewer" do
  version "{VERSION}"

  on_intel do
    sha256 "{MACOS_X86_64_SHA256}"
    url "https://github.com/s00d/RocksDBFusion/releases/download/app-v{VERSION}/rocksdb-viewer_{VERSION}_x64.dmg"
  end

  on_arm do
    sha256 "{MACOS_AARCH64_SHA256}"
    url "https://github.com/s00d/RocksDBFusion/releases/download/app-v{VERSION}/rocksdb-viewer_{VERSION}_aarch64.dmg"
  end

  name "RocksDB Viewer"
  desc "A simple Tauri application to view and interact with a RocksDB database"
  homepage "https://github.com/s00d/RocksDBFusion"

  app "rocksdb-viewer.app"

  postflight do
    system_command "/usr/bin/xattr",
                   args: ["-cr", "#{staged_path}/rocksdb-viewer.app"],
                   sudo: true

    system_command "/usr/bin/codesign",
                   args: ["--force", "--deep", "--sign", "-", "#{staged_path}/rocksdb-viewer.app"],
                   sudo: true
  end

  zap trash: [
    "~/Library/Preferences/com.rocksdb.viewer.plist",
    "~/Library/Saved Application State/com.rocksdb.viewer.savedState",
  ]
end
