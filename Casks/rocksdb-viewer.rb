cask "rocksdb-viewer" do
  version "0.0.2"

  on_intel do
    sha256 "928366c12fee19915e9897c8154860c4cb45769046c569f2a3f8ec95461cdc4b"
    url "https://github.com/s00d/RocksDBFusion/releases/download/app-v0.0.2/rocksdb-viewer_0.0.2_x64.dmg"
  end

  on_arm do
    sha256 "a59431cb85d17996897b84ec24d1c7caa33bd4ff8a69631d348df9cb943467ff"
    url "https://github.com/s00d/RocksDBFusion/releases/download/app-v0.0.2/rocksdb-viewer_0.0.2_aarch64.dmg"
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

  caveats <<~EOS
    During the installation process, you will be prompted to enter your password.
    This is necessary to clear extended attributes and to self-sign the application
    using the `xattr` and `codesign` commands to ensure it runs correctly on macOS.
  EOS
end
