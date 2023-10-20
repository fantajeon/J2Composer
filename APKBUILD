pkgname=jintemplify
pkgver=0.1.8
pkgrel=0
pkgdesc="Template tool generating formats using Jinja2 & YAML"
url="https://github.com/fantajeon/jintemplify"
arch="all"
license="MIT"
depends=""
makedepends="cargo"
install=""
subpackages=""
source="$pkgname-$pkgver.tar.gz::https://github.com/fantajeon/jintemplify/archive/refs/tags/v$pkgver.tar.gz"

build() {
    cd "$builddir"
    cargo build --release
}

package() {
    cd "$builddir"
    install -Dm755 target/release/jintemplify "$pkgdir/usr/bin/jintemplify"
}

sha512sums=""