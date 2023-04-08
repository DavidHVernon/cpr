#!/bin/bash
set -x
set -e
cargo build --release
pushd ./target/release
if test -f "cpr.zip"; then
    rm cpr.zip
fi
codesign -s  "Developer ID Application: David Vernon (3CT7AJ22D9)" --options=runtime cpr
zip cpr.zip cpr
codesign -s  "Developer ID Application: David Vernon (3CT7AJ22D9)" --options=runtime cpr.zip
xcrun notarytool submit cpr.zip  --keychain-profile "rust-notarize-app" --wait
popd
