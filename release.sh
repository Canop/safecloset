# build a new release of safecloset
# This isn't used for normal compilation but for the building of the official releases
version=$(./version.sh)

echo "Building release $version"

# make the build directory and compile for all targets
./compile-all-targets.sh

# add the readme and changelog in the build directory
echo "This is safecloset. More info and installation instructions on https://github.com/Canop/safecloset" > build/README.md
cp CHANGELOG.md build

# publish version number
echo "$version" > build/version

# prepare the release archive
rm safecloset_*.zip
zip -r "safecloset_$version.zip" build/*

# copy it to releases folder
mkdir releases
cp "safecloset_$version.zip" releases
