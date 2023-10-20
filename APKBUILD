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
source="$pkgname-$pkgver::git+https://github.com/fantajeon/jintemplify.git#tag=v$pkgver"

build() {
    echo "try build ${builddir}"
    [ -f "/workspace/Cargo.toml" ] && rm "/workspace/Cargo.toml"
    cd "$builddir"
    cargo build --release
}

package() {
    echo "try package $pkgdir"
    cd "$builddir"
    install -Dm755 target/release/jintemplify "$pkgdir/usr/bin/jintemplify"
}

sha512sums="SKIP"