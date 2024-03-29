#!/usr/bin/env bash

# this doesn't actually work right but it would be cool if it did

# nix-shell -p dig socat --run "./fetch-recent-fast.sh"

set -ex

# rK73XJC40RoiBN4LGOKh - last from data
bet=rK73XJC40RoiBN4LGOKh
idx=0

pending_socket=/tmp/mffetch-init-$RANDOM
socat openssl-connect:api.manifold.markets:443 UNIX-LISTEN:$pending_socket &

while true
do
    file="/tmp/recent-bets/$(printf '%010d' $idx).json"
    echo doing $idx $bet $file

    while [ ! -S $pending_socket ]
    do
        sleep 0.1
    done

    socket=$pending_socket # the old $socket is closed now
    pending_socket=/tmp/mffetch-$idx-$RANDOM # prepare next socket
    socat openssl-connect:api.manifold.markets:443 UNIX-LISTEN:$pending_socket 2> /dev/null &

    curl -s --http1.1 --unix-socket $socket -H "Connection: keep-alive" "http://api.manifold.markets:443/v0/bets?order=asc&after=$bet" > $file

    idx=$((idx + 1))
    bet=$(jq -r '.[-1].id' < $file)
    if test ${#bet} -lt 10; then echo done; exit 0; fi
done
