#!/bin/sh
VERSION=`cat Cargo.toml | head | grep version | sed 's/version = "\(.*\)"/\1/g'`
echo $VERSION

cargo rpm build
mv target/release/rpmbuild/RPMS/x86_64/timer-for-harvest-$VERSION-1.x86_64.rpm fedora-34-timer-for-harvest-$VERSION-1.x86_64.rpm

IMAGE=`podman build -f Dockerfile.debian-buster | tail -n1`
podman run --rm --entrypoint /bin/sh $IMAGE -c "cat /home/user/target/debian/timer-for-harvest_${VERSION}_amd64.deb" > timer-for-harvest_${VERSION}_amd64.deb

sha256sum *rpm *deb > SHA256SUM
