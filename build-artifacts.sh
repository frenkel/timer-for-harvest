#!/bin/sh
VERSION=`cat Cargo.toml | head | grep version | sed 's/version = "\(.*\)"/\1/g'`
echo $VERSION

cargo rpm build
mv target/release/rpmbuild/RPMS/x86_64/timer-for-harvest-$VERSION-1.x86_64.rpm fedora-33-timer-for-harvest-$VERSION-1.x86_64.rpm

IMAGE=`podman build -f Dockerfile.debian-buster | tail -n1`
podman run --rm --entrypoint /bin/sh $IMAGE -c "cat /home/user/target/debian/timer-for-harvest_${VERSION}_amd64.deb" > debian-10-timer-for-harvest_${VERSION}_amd64.deb

IMAGE=`podman build -f Dockerfile.ubuntu-focal | tail -n1`
podman run --rm --entrypoint /bin/sh $IMAGE -c "cat /home/user/target/debian/timer-for-harvest_${VERSION}_amd64.deb" > ubuntu-20.04-timer-for-harvest_${VERSION}_amd64.deb

IMAGE=`podman build -f Dockerfile.ubuntu-groovy | tail -n1`
podman run --rm --entrypoint /bin/sh $IMAGE -c "cat /home/user/target/debian/timer-for-harvest_${VERSION}_amd64.deb" > ubuntu-20.10-timer-for-harvest_${VERSION}_amd64.deb

sha256sum *rpm *deb > SHA256SUM
