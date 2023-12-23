ADDRESS=erd1qqqqqqqqqqqqqpgqa4smstepaxfq0s5fxrwg45l64cz5mwl4ah0svfsusq #Template SC

DEPLOY_TRANSACTION=$(mxpy data load --key=deployTransaction-devnet)
PROXY=https://devnet-api.multiversx.com
PROJECT="${PWD}/output/inscriptions.wasm"

deploy() {
    echo ${PROJECT}
    mxpy --verbose contract deploy --metadata-payable --bytecode=${PROJECT} --recall-nonce  --gas-limit=50000000 \
    --send --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=0 --proxy=${PROXY} --chain="D" || return
}

upgrade() {
    mxpy --verbose contract upgrade ${ADDRESS} --metadata-payable --bytecode=${PROJECT} --recall-nonce --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=0 \
    --gas-limit=50000000 --send --outfile="upgrade.json" --proxy=${PROXY} --chain="D" || return
}

issue() {
    mxpy --verbose contract call ${ADDRESS} --function="issue" --value 50000000000000000 --arguments str:Inscriptions str:INS --recall-nonce --ledger --ledger-account-index=0 --ledger-address-index=0 \
    --gas-limit=90000000 --send --proxy=${PROXY} --chain="D" || return
}