# Maintainer: Byte Strix <contact@bytestrix.com>
pkgname=cliai
pkgver=0.1.0
pkgrel=1
pkgdesc="A completely free and open-source CLI assistant powered by AI - use your own API keys or run locally"
arch=('x86_64' 'aarch64')
url="https://github.com/cliai-team/cliai"
license=('MIT')
depends=('gcc-libs' 'openssl')
makedepends=('rust')
optdepends=('ollama: Local AI backend')
source=("$pkgname-$pkgver.tar.gz::https://github.com/cliai-team/$pkgname/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
  cd "$pkgname-$pkgver"
  cargo build --release --locked
}

package() {
  cd "$pkgname-$pkgver"
  install -Dm755 "target/release/cliai" "$pkgdir/usr/bin/cliai"
}
