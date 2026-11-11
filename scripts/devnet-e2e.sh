#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
endpoint="${LYQUOR_WS:-ws://127.0.0.1:10087/ws}"
rpc="${LYQUOR_RPC:-http://127.0.0.1:10087/api}"
ldk_dir="${LYQUOR_LDK_DIR:-$HOME/.shakenup/ldk}"
if [[ ! -f "$ldk_dir/lyquid/Cargo.toml" ]]; then
    echo "Lyquid SDK missing at $ldk_dir. Set LYQUOR_LDK_DIR to the installed LDK directory." >&2
    exit 1
fi
ldk_dir="$(cd "$ldk_dir" && pwd -P)"
stage="$(mktemp -d)"
trap 'rm -rf -- "$stage"' EXIT
cp -R "$root/lyquid/src" "$stage/src"
sdk_path="$(jq -Rn --arg path "$ldk_dir/lyquid" '$path')"
core_path="$(jq -Rn --arg path "$root" '$path')"
while IFS= read -r line; do
    case "$line" in
        'lyquid = '*)
            printf 'lyquid = { path = %s, features = ["ldk"] }\n' "$sdk_path" ;;
        'club-reconcile = '*)
            printf 'club-reconcile = { path = %s }\n' "$core_path" ;;
        *)
            printf '%s\n' "$line" ;;
    esac
done < "$root/lyquid/Cargo.toml" > "$stage/Cargo.toml"

: "${SIGNER_A:?set SIGNER_A to a funded disposable devnet private key}"
: "${SIGNER_B:?set SIGNER_B to a funded disposable devnet private key}"
: "${SIGNER_C:?set SIGNER_C to a funded disposable devnet private key}"
a=$(cast wallet address --private-key "$SIGNER_A")
b=$(cast wallet address --private-key "$SIGNER_B")
c=$(cast wallet address --private-key "$SIGNER_C")
init=$(cast abi-encode 'constructor(string,string,string)' "$a" "$b" "$c")
deployed=$(shaker deploy --endpoint "$endpoint" --input "$init" "$stage/Cargo.toml")
printf '%s\n' "$deployed"
contract=$(printf '%s\n' "$deployed" | sed -nE 's/^Contract:[[:space:]]+(0x[0-9a-fA-F]{40})[[:space:]]*$/\1/p' | tail -1)
test -n "$contract"
if ! printf '%s\n' "$deployed" | grep -Eq '^Lyquid ID:[[:space:]]+Lyquid-[[:alnum:]]+[[:space:]]*$'; then
    printf 'Contract %s was created, but the node has not resolved its Lyquid ID. Check the localnet node log.\n' "$contract" >&2
    exit 1
fi
compute() {
    local encoded
    encoded=$(cast call --rpc-url "$rpc" "$contract" 'compute(string)' "$(cat "$root/fixtures/$1.csv")") || return
    cast abi-decode 'f()(string)' "$encoded" --json | jq -er '.[0]'
}
read_result() {
    local encoded
    encoded=$(cast call --rpc-url "$rpc" "$contract" 'result()') || return
    cast abi-decode 'f()(string)' "$encoded" --json | jq -er '.[0]'
}
submit() {
    cast send --rpc-url "$rpc" --private-key "$1" "$contract" 'submit(string)' "$2" >/dev/null
}
first=$(compute complete)
second=$(compute missing)
third=$(compute reordered)
[[ "$first" != "$second" && "$first" == "$third" ]]
submit "$SIGNER_A" "$first"
submit "$SIGNER_B" "$second"
count=0
for ((i=0; i<30; i++)); do
    count=$(cast call --rpc-url "$rpc" "$contract" 'vote_count()(uint64)')
    if [[ "$count" == 2 ]]; then
        break
    fi
    sleep 1
done
[[ "$count" == 2 ]]
test -z "$(read_result)"
echo 'A + B: PENDING'
submit "$SIGNER_C" "$third"
for ((i=0; i<30; i++)); do
    if [[ "$(read_result)" == "$first" ]]; then
        printf 'A + B + C: PUBLISHED\n%s\n' "$first"
        exit 0
    fi
    sleep 1
done
echo 'Timed out waiting for publication' >&2
exit 1
