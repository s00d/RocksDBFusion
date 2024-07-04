import fs from "fs";
import path from "path";

function getVersionFromCargoToml(filePath) {
    const content = fs.readFileSync(filePath, 'utf-8');
    const match = content.match(/^version\s*=\s*"([^"]+)"$/m);
    return match ? match[1] : null;
}

function getVersionFromJson(filePath) {
    const content = fs.readFileSync(filePath, 'utf-8');
    const json = JSON.parse(content);
    return json.package.version;
}

function getVersions() {
    const serverVersion = getVersionFromCargoToml(path.resolve(__dirname, '../../../server/Cargo.toml'));
    const cliVersion = getVersionFromCargoToml(path.resolve(__dirname, '../../../rocksdb-cli/Cargo.toml'));
    const appVersion = getVersionFromJson(path.resolve(__dirname, '../../../rocksdb-viewer/src-tauri/tauri.conf.json'));

    return [
        {
            text: `Server (v${serverVersion})`,
            children: [
                {
                    text: `MacOS x86_64`,
                    link: `https://github.com/s00d/RocksDBFusion/releases/download/server-v${serverVersion}/rocksdb_server-Darwin-x86_64.tar.gz`,
                },
                {
                    text: `MacOS Aarch64`,
                    link: `https://github.com/s00d/RocksDBFusion/releases/download/server-v${serverVersion}/rocksdb_server-Darwin-aarch64.tar.gz`,
                },
                {
                    text: `Linux x86_64`,
                    link: `https://github.com/s00d/RocksDBFusion/releases/download/server-v${serverVersion}/rocksdb_server-Linux-x86_64-musl.tar.gz`,
                },
                {
                    text: `Linux Aarch64`,
                    link: `https://github.com/s00d/RocksDBFusion/releases/download/server-v${serverVersion}/rocksdb_server-Linux-aarch64-musl.tar.gz`,
                },
                {
                    text: `Linux i686`,
                    link: `https://github.com/s00d/RocksDBFusion/releases/download/server-v${serverVersion}/rocksdb_server-Linux-i686-musl.tar.gz`,
                },

                {
                    text: `Windows aarch64`,
                    link: `https://github.com/s00d/RocksDBFusion/releases/download/server-v${serverVersion}/rocksdb_server-Windows-aarch64.zip`,
                },
                {
                    text: `Windows x86_64`,
                    link: `https://github.com/s00d/RocksDBFusion/releases/download/server-v${serverVersion}/rocksdb_server-Windows-x86_64.zip`,
                },
            ],
        },
        {
            text: `CLI (v${cliVersion})`,
            children: [
                {
                    text: `MacOS x86_64`,
                    link: `https://github.com/s00d/RocksDBFusion/releases/download/rocksdb-cli-v${cliVersion}/rocksdb_cli-Darwin-x86_64.tar.gz`,
                },
                {
                    text: `MacOS Aarch64`,
                    link: `https://github.com/s00d/RocksDBFusion/releases/download/rocksdb-cli-v${cliVersion}/rocksdb_cli-Darwin-aarch64.tar.gz`,
                },
                {
                    text: `Linux x86_64`,
                    link: `https://github.com/s00d/RocksDBFusion/releases/download/rocksdb-cli-v${cliVersion}/rocksdb_cli-Linux-x86_64-musl.tar.gz`,
                },
                {
                    text: `Linux Aarch64`,
                    link: `https://github.com/s00d/RocksDBFusion/releases/download/rocksdb-cli-v${cliVersion}/rocksdb_cli-Linux-aarch64-musl.tar.gz`,
                },
                {
                    text: `Linux i686`,
                    link: `https://github.com/s00d/RocksDBFusion/releases/download/rocksdb-cli-v${cliVersion}/rocksdb_cli-Linux-i686-musl.tar.gz`,
                },

                {
                    text: `Windows aarch64`,
                    link: `https://github.com/s00d/RocksDBFusion/releases/download/rocksdb-cli-v${cliVersion}/rocksdb_cli-Windows-aarch64.zip`,
                },
                {
                    text: `Windows x86_64`,
                    link: `https://github.com/s00d/RocksDBFusion/releases/download/rocksdb-cli-v${cliVersion}/rocksdb_cli-Windows-x86_64.zip`,
                },
            ],
        },
        {
            text: `Viewer (v${appVersion})`,
            children: [
                {
                    text: `MacOS x86_64`,
                    link: `https://github.com/s00d/RocksDBFusion/releases/download/app-v${appVersion}/rocksdb-viewer_${appVersion}_x64.dmg`,
                },
                {
                    text: `MacOS Aarch64`,
                    link: `https://github.com/s00d/RocksDBFusion/releases/download/app-v${appVersion}/rocksdb-viewer_${appVersion}_aarch64.dmg`,
                },
                {
                    text: `Linux x86_64`,
                    link: `https://github.com/s00d/RocksDBFusion/releases/download/app-v${appVersion}/rocksdb-viewer_${appVersion}_amd64.deb`,
                },
                {
                    text: `Windows x86_64`,
                    link: `https://github.com/s00d/RocksDBFusion/releases/download/app-v${appVersion}/rocksdb-viewer_${appVersion}_x64-setup.exe`,
                },
            ],
        },
    ];
}

export default getVersions
