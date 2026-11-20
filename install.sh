#!/bin/sh
set -eu

repository=Rabin18Glan/Tictactoe
install_dir=${INSTALL_DIR:-"$HOME/.local/bin"}
version=${VERSION:-latest}

fail() { printf '%s\n' "Error: $*" >&2; exit 1; }
for tool in curl tar mktemp; do
    command -v "$tool" >/dev/null 2>&1 || fail "$tool is required"
done
case "$(uname -s):$(uname -m)" in
    Linux:x86_64) target=x86_64-unknown-linux-gnu ;;
    Darwin:arm64|Darwin:aarch64) target=aarch64-apple-darwin ;;
    *) fail "No release binary for this platform. Use: cargo install --git https://github.com/$repository.git --locked" ;;
esac
asset="tictactoe-$target.tar.gz"
if [ "$version" = latest ]; then
    base="https://github.com/$repository/releases/latest/download"
else
    case "$version" in
        v[0-9]* ) ;;
        *) fail 'VERSION must be a release tag such as v0.1.0' ;;
    esac
    case "$version" in *[!a-zA-Z0-9._-]*) fail 'Invalid release tag' ;; esac
    base="https://github.com/$repository/releases/download/$version"
fi
work_dir=$(mktemp -d)
trap 'rm -rf "$work_dir"' EXIT HUP INT TERM
printf 'Downloading %s (%s)…\n' "$asset" "$version"
curl --fail --location --silent --show-error --proto '=https' "$base/$asset" -o "$work_dir/$asset" || fail 'Download failed. Check that a binary release has been published.'
curl --fail --location --silent --show-error --proto '=https' "$base/SHA256SUMS" -o "$work_dir/SHA256SUMS"
# Releases contain paths such as artifacts/<asset>; compare only this asset.
expected=$(awk -v asset="$asset" '{ n=split($2,p,"/"); if(p[n]==asset) print $1 }' "$work_dir/SHA256SUMS")
[ "${#expected}" = 64 ] || fail 'Missing or invalid release checksum'
if command -v sha256sum >/dev/null 2>&1; then
    actual=$(sha256sum "$work_dir/$asset" | awk '{print $1}')
elif command -v shasum >/dev/null 2>&1; then
    actual=$(shasum -a 256 "$work_dir/$asset" | awk '{print $1}')
else
    fail 'sha256sum or shasum is required'
fi
[ "$actual" = "$expected" ] || fail 'Checksum mismatch'
tar -xzf "$work_dir/$asset" -C "$work_dir" tictactoe
mkdir -p "$install_dir"
install -m 755 "$work_dir/tictactoe" "$install_dir/tictactoe"
printf 'Installed %s/tictactoe\n' "$install_dir"
case ":${PATH:-}:" in
    *":$install_dir:"*) printf 'Run: tictactoe\n' ;;
    *) printf 'Add to your shell profile: export PATH="%s:$PATH"\n' "$install_dir" ;;
esac
