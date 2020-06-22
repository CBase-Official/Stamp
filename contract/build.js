const shell = require('shelljs');

shell.fatal = true; // same as "set -e"

shell.cd('contract');
// Note: see flags in ./cargo/config
shell.exec('RUSTFLAGS=\'-C link-arg=-s\' cargo build --target wasm32-unknown-unknown --release');
shell.cp('./target/wasm32-unknown-unknown/release/stamp.wasm', './res');