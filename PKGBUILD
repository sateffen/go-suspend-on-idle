pkgname=go-suspend-on-idle
pkgver=1.0.0
pkgrel=1
pkgdesc="Go Suspend on Idle"
depends=('iproute2')
makedepends=('go' 'make')
arch=('x86_64' 'aarch64')
url="https://github.com/sateffen/go-suspend-on-idle"
license=('MIT')
source=('Makefile' 'main.go' 'isanyuseractive.go' 'isnetworkactive.go' 'go.mod' 'go-suspend-on-idle.service')

sha256sums=('SKIP' 'SKIP' 'SKIP' 'SKIP' 'SKIP' 'SKIP')

prepare() {
  export GOPATH="$srcdir/build"
  export GOFLAGS="-buildmode=pie -trimpath -ldflags=-linkmode=external -mod=readonly -modcacherw"

  go mod download
}

build() {
  export GOPATH="$srcdir/build"
  export CGO_CPPFLAGS="${CPPFLAGS}"
  export CGO_CFLAGS="${CFLAGS}"
  export CGO_CXXFLAGS="${CXXFLAGS}"
  export CGO_LDFLAGS="${LDFLAGS}"
  export GOFLAGS="-buildmode=pie -trimpath -ldflags=-linkmode=external -mod=readonly -modcacherw"

  make build
}

package() {
  install -Dm755 "$srcdir/bin/go-suspend-on-idle" "$pkgdir/usr/bin/go-suspend-on-idle"
  install -Dm644 "$srcdir/go-suspend-on-idle.service" "$pkgdir/usr/lib/systemd/system/go-suspend-on-idle.service"
}