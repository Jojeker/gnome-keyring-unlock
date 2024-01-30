pkgname=gnome-keyring-unlock
pkgver=0.1
pkgrel=1
pkgdesc="Unlock the gnome keyring from a keyfile."
arch=('any')
url="https://git.ve0.cc/eduard/gnome-keyring-unlock"
license=('GPL-3.0')
depends=('cargo')
makedepends=('cargo' 'gcc')
source=("${pkgname}-${pkgver}.tar.gz::https://git.ve0.cc/eduard/gnome-keyring-unlock/archive/v${pkgver}.tar.gz")
sha256sums=('f69badb942a3d27e5321d0ee31a0f09c94e44bd67ea5d58b25e9311bbf58b8e3')

build() {
    cd "$srcdir/$pkgname"
    pwd
    cargo build --release
}

package() {
    cd "$srcdir/$pkgname"
    
    # Install binary and service file
    cargo install --root="$pkgdir" --path .
    install -Dm644 "$srcdir/$pkgname/config/gkd-unlock.service" "$pkgdir/usr/lib/systemd/system/gkd-unlock.service"

    # Copy sample config
    install -Dm644 "$srcdir/$pkgname/config/gkd-unlock.yaml" "$pkgdir/etc/gkd-unlock/config.yaml"

    # Remove unnecessary files
    rm -f $pkgdir/.crates2.json
    rm -f $pkgdir/.crates.toml
}
