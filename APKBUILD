pkgname=jintemplify
pkgver=0.1.3 # 현재 프로젝트 버전에 따라 업데이트
pkgrel=0
pkgdesc="jintemplify is a versatile tool that enables users to combine Jinja2 templates with YAML variables, producing files in any desired format. The application also supports a plugin system based on shell scripts, allowing users to extend its functionality with familiar scripting techniques."
url="https://github.com/fantajeon/jintemplify"
arch="all"
license="MIT"
depends=""
makedepends="cargo"
install=""
subpackages=""
source="$pkgname-$pkgver.tar.gz::https://github.com/fantajeon/jintemplify/archive/refs/tags/$pkgver.tar.gz"

build() {
    cd "$builddir"
    cargo build --release
}

package() {
    cd "$builddir"
    install -Dm755 target/release/jintemplify "$pkgdir/usr/bin/jintemplify"
}