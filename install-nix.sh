#!/usr/bin/env bash

echo "Initialize Go environment ..."
if ! command -v gvm 2>&1 > /dev/null; then
  echo "Install GVM ..."
  cargo install --path .
fi

gvm init
gvm update

echo ""
echo "Install latest stable version ..."
latest_version="$(gvm ls-remote --stable | tail -n1 | awk '{print $1}')"
gvm install $latest_version --use

echo ""
echo "Reload profile ..."
source ~/.profile
