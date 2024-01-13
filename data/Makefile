all: markets.json bets
.PHONY: all

markets.zip:
	wget -O markets.zip 'https://firebasestorage.googleapis.com/v0/b/mantic-markets.appspot.com/o/trade-dumps%2Fmanifold-dump-markets-04082023.json.zip?alt=media&token=7e18a376-6ac3-4d66-a9a0-552b967f2fe8'

bets.json:
	wget -O bets.zip 'https://firebasestorage.googleapis.com/v0/b/mantic-markets.appspot.com/o/trade-dumps%2Fmanifold-dump-bets-04082023.zip?alt=media&token=c3ffdfbd-6769-48e3-8cfc-4fc93c2443f3'

markets.json: markets.zip
	unzip -p markets.zip manifold-dump-markets-04082023.json > markets.json

bets: bets.json
	unzip bets.zip
