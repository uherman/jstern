version=$1
if [ -z "$version" ]; then
  echo "Usage: release.sh <version>"
  exit 1
fi

echo "Building binaries for version $version"
cross build --release --target x86_64-apple-darwin
cross build --release --target aarch64-apple-darwin
CROSS_CONTAINER_OPTS="--platform linux/amd64" cross build --release --target x86_64-unknown-linux-gnu
CROSS_CONTAINER_OPTS="--platform linux/amd64" cross build --release --target arm-unknown-linux-gnueabihf
CROSS_CONTAINER_OPTS="--platform linux/amd64" cross build --release --target aarch64-unknown-linux-gnu
CROSS_CONTAINER_OPTS="--platform linux/amd64" cross build --release --target x86_64-pc-windows-gnu

mkdir -p release/${version}

echo "Creating release artifacts"
tar -czvf release/${version}/jstern_${version}_darwin_amd64.tar.gz -C target/x86_64-apple-darwin/release jstern
tar -czvf release/${version}/jstern_${version}_darwin_arm64.tar.gz -C target/aarch64-apple-darwin/release jstern
tar -czvf release/${version}/jstern_${version}_linux_amd64.tar.gz -C target/x86_64-unknown-linux-gnu/release jstern
tar -czvf release/${version}/jstern_${version}_linux_arm.tar.gz -C target/arm-unknown-linux-gnueabihf/release jstern
tar -czvf release/${version}/jstern_${version}_linux_arm64.tar.gz -C target/aarch64-unknown-linux-gnu/release jstern
tar -czvf release/${version}/jstern_${version}_windows_amd64.tar.gz -C target/x86_64-pc-windows-gnu/release jstern.exe
zip release/${version}/jstern_${version}_windows_amd64.zip target/x86_64-pc-windows-gnu/release/jstern.exe

echo "Creating checksums.txt"
shasum -a 256 release/${version}/jstern_${version}_darwin_amd64.tar.gz > release/${version}/checksums.txt
shasum -a 256 release/${version}/jstern_${version}_darwin_arm64.tar.gz >> release/${version}/checksums.txt
shasum -a 256 release/${version}/jstern_${version}_linux_amd64.tar.gz >> release/${version}/checksums.txt
shasum -a 256 release/${version}/jstern_${version}_linux_arm.tar.gz >> release/${version}/checksums.txt
shasum -a 256 release/${version}/jstern_${version}_linux_arm64.tar.gz >> release/${version}/checksums.txt
shasum -a 256 release/${version}/jstern_${version}_windows_amd64.tar.gz >> release/${version}/checksums.txt
shasum -a 256 release/${version}/jstern_${version}_windows_amd64.zip >> release/${version}/checksums.txt

# Extract only the filenames
sed -i '' "s|release/${version}/||g" release/${version}/checksums.txt

echo "Done!"