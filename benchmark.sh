


echo "Start benchmarking..."

# Generate test code
cargo run --example generate
cd ./temp/

hyperfine --warmup 3 'lok' 'tokei'

cd ../
rm -rf ./temp/


