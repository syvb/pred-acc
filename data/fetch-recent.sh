#!/usr/bin/env bash

set -ex

# rK73XJC40RoiBN4LGOKh - last from data
bet=rK73XJC40RoiBN4LGOKh
idx=0

while true
do
    file="recent-bets/$(printf '%010d' $idx).json"
    wget -q --show-progress -O $file "https://api.manifold.markets/v0/bets?order=asc&after=$bet" 
    sleep 0.06 # rate limit (not really needed since requests are sequential)

    idx=$((idx + 1))
    bet=$(jq -r '.[-1].id' < $file)
    if test ${#bet} -lt 10; then echo done; exit 0; fi
done
