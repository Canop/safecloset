# Reference closets

Those closet files are read by the tests in `src/core/compat_test.rs`, which
check that a closet written by an older version of SafeCloset is still
readable, and stays intact when saved by the current version.

They hold no secret: their passphrases are in `compat_test.rs`.

## Don't write to those files

Saving a closet renames its file to `*.old` and rewrites it in the current
format. Doing that to a reference closet would replace it with a current one,
and the tests would stay green whatever happens afterwards.

The tests always work on a copy in a temporary directory. Don't open those
files with safecloset itself either.

## Adding one

    cargo test generate_reference_closet -- --ignored --nocapture

It writes `<version>.closet` here, and fails if the file already exists.
There's nothing to declare: every `*.closet` file of this directory is
discovered and checked.

