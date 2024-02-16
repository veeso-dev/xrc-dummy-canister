#!/bin/bash

set -e

deploy_canister() {
  INSTALL_MODE="$1"
  NETWORK="$2"
  PRINCIPAL="$3"
  NAME="$4"
  SYMBOL="$5"
  LOGO="$6"
  FEE="$7"
  DECIMALS="$8"
  TOTAL_SUPPLY="$9"
  ACCOUNTS="${10}"
  MINTING_ACCOUNT="${11}"

  echo "deploying $NAME canister $PRINCIPAL"

  init_args="(record {
    admins = vec { $(for admin in $ADMINS; do echo "principal \"$admin\";"; done) };
    total_supply = $TOTAL_SUPPLY;
    accounts = $ACCOUNTS;
    minting_account = $MINTING_ACCOUNT;
    name = \"$NAME\";
    symbol = \"$SYMBOL\";
    logo = \"$LOGO\";
    fee = $FEE;
    decimals = $DECIMALS;
  })"

  dfx deploy --mode=$INSTALL_MODE --yes --network="$NETWORK" --argument="$init_args" icrc2-template

}
