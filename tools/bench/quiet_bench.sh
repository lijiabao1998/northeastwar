#!/bin/bash
# Run northeast --bench <dir> from the release build once no headless test browser is running.
# Usage: quiet_bench.sh <out-dir> [exe]
cd /c/RUST/northeastwar
out=$1
exe=${2:-./target/x86_64-pc-windows-gnullvm/release/northeast.exe}
busy() {
  powershell -NoProfile -Command "@(Get-CimInstance Win32_Process -Filter \"Name='chrome.exe'\" | Where-Object { \$_.CommandLine -match '--headless' }).Count" | tr -d '\r'
}
waited=0
while [ "$(busy)" != "0" ] && [ $waited -lt 1200 ]; do
  sleep 15
  waited=$((waited + 15))
done
mkdir -p "$(dirname "$out")"
before=$(busy)
start=$(date +%s)
"$exe" --bench "$out" > "$out.log" 2>&1
code=$?
echo "exit $code after $(( $(date +%s) - start ))s; waited ${waited}s; headless browsers before $before, after $(busy)"
cat "$out/bench.txt"
