#!/bin/bash

cd "$(dirname "$0")" || exit 1

CANISTER_IDS="../.dfx/local/canister_ids.json"
PRINCIPAL="$(cat "$CANISTER_IDS" | jq -r '.icrc2-template-canister.local')"

source ./deploy_functions.sh
source ./did.sh

ADMIN_PRINCIPAL="$(dfx identity get-principal)"
TOTAL_SUPPLY="8880101010000000000"
ACCOUNTS="$(balances "$ADMIN_PRINCIPAL:250000000000000000")"
MINTING_ACCOUNT="$(account "$ADMIN_PRINCIPAL" "{33;169;149;73;231;146;144;124;94;39;94;84;81;6;141;173;223;77;67;238;141;202;180;135;86;35;26;143;183;113;49;35}")"

dfx stop
dfx start --background

cd ../

deploy_canister "reinstall" "local" \
  "$PRINCIPAL" \
  "MyToken" \
  "MYT" \
  "https://raw.githubusercontent.com/dfinity-lab/token-registry/main/assets/dfinity.png" \
  "0" \
  "8" \
  "$TOTAL_SUPPLY" \
  "$ACCOUNTS" \
  "$MINTING_ACCOUNT"

dfx stop

exit $RES
