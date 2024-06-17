cd ./rocksdb-client-node
npm run test
npm run build
npm publish
cd ..
cargo publish --manifest-path=rocksdb-client-rust/Cargo.toml
cd ./rocksdb-client-python
python -m unittest discover -s tests
python setup.py sdist bdist_wheel
twine upload dist/*