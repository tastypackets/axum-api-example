#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

cert_dir=".dev/certs"
cert="$cert_dir/localhost.pem"
key="$cert_dir/localhost-key.pem"

generate() {
  if [[ -f "$cert" && -f "$key" ]]; then
    openssl x509 -in "$cert" -checkend 0 -noout || {
      echo "Certificate expired. Uninstall it, remove .dev/certs, and run mise run setup." >&2
      exit 1
    }
    echo "Using existing $cert"
    return
  fi
  if [[ -e "$cert" || -e "$key" ]]; then
    echo "Incomplete certificate pair. Restore the missing file or remove .dev/certs and run setup." >&2
    exit 1
  fi

  umask 077
  mkdir -p .dev
  local staging
  staging=$(mktemp -d .dev/certs.XXXXXX)
  trap 'rm -rf "$staging"' EXIT
  cat > "$staging/openssl.cnf" <<'EOF'
[req]
prompt = no
distinguished_name = subject
x509_extensions = extensions
[subject]
CN = API Example Local Development
[extensions]
basicConstraints = critical, CA:FALSE
keyUsage = critical, digitalSignature, keyEncipherment
extendedKeyUsage = serverAuth
subjectAltName = DNS:localhost, IP:127.0.0.1, IP:::1
EOF
  openssl req -x509 -newkey rsa:2048 -nodes -sha256 -days 365 \
    -config "$staging/openssl.cnf" \
    -keyout "$staging/localhost-key.pem" -out "$staging/localhost.pem" \
    2> "$staging/generate.log" || { cat "$staging/generate.log" >&2; exit 1; }
  mkdir -p "$cert_dir"
  mv "$staging/localhost-key.pem" "$key"
  mv "$staging/localhost.pem" "$cert"
  rm -rf "$staging"
  trap - EXIT
  echo "Generated $cert"
}

require_macos() {
  if [[ "$(uname -s)" != Darwin ]]; then
    echo "This task requires macOS." >&2
    exit 1
  fi
}

case "${1:-}" in
  generate) generate ;;
  install-macos)
    require_macos
    generate
    security add-trusted-cert -r trustRoot -p ssl \
      -k "$HOME/Library/Keychains/login.keychain-db" "$cert"
    echo "Trusted $cert in the user login keychain"
    ;;
  uninstall-macos)
    require_macos
    if [[ ! -f "$cert" ]]; then
      echo "No local certificate found. Keep .dev/certs until after uninstalling trust." >&2
      exit 1
    fi
    fingerprint=$(openssl x509 -in "$cert" -noout -fingerprint -sha256)
    fingerprint=${fingerprint#*=}
    fingerprint=${fingerprint//:/}
    security delete-certificate -t -Z "$fingerprint" \
      "$HOME/Library/Keychains/login.keychain-db"
    echo "Removed the certificate and its user trust. Local PEM files are kept."
    ;;
  *) echo "Usage: $0 generate|install-macos|uninstall-macos" >&2; exit 1 ;;
esac
