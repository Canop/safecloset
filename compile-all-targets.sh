# WARNING: This script is NOT meant for normal installation, it's dedicated
# to the compilation of all supported targets, from a linux machine.
# This is a long process and it involves specialized toolchains.
# For usual compilation do
#     cargo build --release

H1="\n\e[30;104;1m\e[2K\n\e[A" # style first header
H2="\n\e[30;104m\e[1K\n\e[A" # style second header
EH="\e[00m\n\e[2K" # end header
NAME=safecloset
version=$(./version.sh)

echo -e "${H1}Compilation of all targets for $NAME $version${EH}"
 
# clean previous build
rm -rf build
mkdir build
echo "   build cleaned"

# Build versions for other platforms using cargo cross
cross_build() {
    target_name="$1"
    target="$2"
    features="$3"
    echo -e "${H2}Compiling the $target_name version (target=$target, features='$features')${EH}"
    cargo clean
    if [[ -n $features ]]
    then
        cross build --target "$target" --release --features "$features"
    else
        cross build --target "$target" --release
    fi
    mkdir "build/$target"
    if [[ $target_name == 'Windows' ]]
    then
        exec="$NAME.exe"
    else
        exec="$NAME"
    fi
    cp "target/$target/release/$exec" "build/$target/"
}

# cross_build "x86-64 GLIBC" "x86_64-unknown-linux-gnu" ""
# cross_build "MUSL" "x86_64-unknown-linux-musl" ""
# cross_build "ARM 32" "armv7-unknown-linux-gnueabihf" ""
# cross_build "ARM 32 MUSL" "armv7-unknown-linux-musleabi" ""
# cross_build "ARM 64" "aarch64-unknown-linux-gnu" ""
# cross_build "ARM 64 MUSL" "aarch64-unknown-linux-musl" ""
cross_build "Windows" "x86_64-pc-windows-gnu" "clipboard"
cross_build "Android" "aarch64-linux-android" "clipboard"

# Build the default X86_64 linux version (with clipboard support, needing a recent GLIBC)
# recent glibc
echo -e "${H2}Compiling the standard linux version${EH}"
cargo build --release --features "clipboard"
strip "target/release/$NAME"
mkdir build/x86_64-linux/
cp "target/release/$NAME" build/x86_64-linux/

# add a summary of content
echo '
This archive contains pre-compiled binaries.

For more information, or if you prefer to compile yourself, see https://dystroy.org/safecloset/install
' > build/install.md

echo -e "${H1}FINISHED${EH}"
