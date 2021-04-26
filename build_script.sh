#!/bin/sh

#build in release

#build linux_x64
cargo build --release --target=x86_64-unknown-linux-gnu
#build linux_i686
cargo build --release --target=i686-unknown-linux-gnu
#build windows_x64
cargo build --release --target=x86_64-pc-windows-gnu
#build windows_i686
cargo build --release --target=i686-pc-windows-gnu

#strip debug symbols

strip target/x86_64-unknown-linux-gnu/release/c21_counter_rs
strip target/i686-unknown-linux-gnu/release/c21_counter_rs
strip target/x86_64-pc-windows-gnu/release/c21_counter_rs.exe
strip target/i686-pc-windows-gnu/release/c21_counter_rs.exe

#upx compression

upx target/x86_64-unknown-linux-gnu/release/c21_counter_rs
upx --force target/i686-unknown-linux-gnu/release/c21_counter_rs
upx --force target/x86_64-pc-windows-gnu/release/c21_counter_rs.exe
upx --force target/i686-pc-windows-gnu/release/c21_counter_rs.exe
#create folder
mkdir dist-x86_64-linux
mkdir dist-x86_64-windows
mkdir dist-i386-linux
mkdir dist-i386-windows
#fetch files from build artifact
mv target/x86_64-unknown-linux-gnu/release/c21_counter_rs dist-x86_64-linux
mv target/i686-unknown-linux-gnu/release/c21_counter_rs dist-i386-linux
mv target/x86_64-pc-windows-gnu/release/c21_counter_rs.exe dist-x86_64-windows
mv target/i686-pc-windows-gnu/release/c21_counter_rs.exe dist-i386-windows
#install files from work folder
cp work/* dist-x86_64-linux
cp work/* dist-i386-linux
cp work/* dist-x86_64-windows
cp work/* dist-i386-windows
#zip folder
zip -r C21CounterX64Linux.zip dist-x86_64-linux
zip -r C21Counteri386Linux.zip dist-i386-linux
zip -r C21CounterX64Windows.zip dist-x86_64-windows
zip -r C21Counteri386Windows.zip dist-i386-windows
