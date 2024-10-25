# This script compiles safecloset for the local system
#
# After compilation, safecloset can be found in target/release
#
# If you're not a developer but just want to install safecloset to use it,
# you'll probably prefer one of the options listed at
#   https://dystroy.org/safecloset/install
#
# The line below can be safely executed on systems which don't
# support sh scripts.

cargo build --release --features "clipboard"

# If the line above didn't work, you may use this one which won't
# have the "clipboard" feature:
#
# cargo build --release --locked
