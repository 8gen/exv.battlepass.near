#!/usr/bin/env bash

set -e
export NEAR_ENV=testnet
OWNER_ID=$1

realpath() {
    path=`eval echo "$1"`
    folder=$(dirname "$path")
    echo $(cd "$folder"; pwd)/$(basename "$path");
}

if [[ $OWNER_ID == "" ]];then
    echo $0 kalloc.testnet
    exit
fi

./build.sh

ROOT=$(dirname $(realpath $0))

if [[ ! -e ${ROOT}/neardev/hall ]]; then
INIT_HALL_TIME=true
rm -rf ${ROOT}/neardev/nft
fi
if [[ ! -e ${ROOT}/neardev/nft ]]; then
INIT_NFT_TIME=true
fi

mkdir -p ${ROOT}/neardev/{nft,hall}
echo "deploy"
near dev-deploy --wasmFile ${ROOT}/res/nft.wasm --projectKeyDirectory ${ROOT}/neardev/nft --initFunction ''
near dev-deploy --wasmFile ${ROOT}/res/hall.wasm --projectKeyDirectory ${ROOT}/neardev/hall --initFunction ''
NFT_CONTRACT=$(cat ${ROOT}/neardev/nft/dev-account)
HALL_CONTRACT=$(cat ${ROOT}/neardev/hall/dev-account)

if [[ $INIT_NFT_TIME ]];then
    echo "init NFT"
    near call ${NFT_CONTRACT} new_default_meta '{"max_supply": 1000, "name": "Exverse pass", "symbol": "EXVPASS"}' --accountId ${OWNER_ID}
fi
if [[ $INIT_HALL_TIME ]];then
    echo "init HALL"
    near call ${HALL_CONTRACT} new '{"nft_account_id": "'${NFT_CONTRACT}'"}'  --accountId ${OWNER_ID}
fi
echo HALL CONTRACT is ${HALL_CONTRACT}
echo NFT CONTRACT is ${NFT_CONTRACT}


