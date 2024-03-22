test:
	(find programs && find tests) | entr -cs 'anchor build && RUST_LOG= yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/*.ts'

build:
	solana-verify build --library-name openbook_twap -b ellipsislabs/solana:1.16.10

deploy CLUSTER:
	solana program deploy -u {{ CLUSTER }} --program-id ./target/deploy/openbook_twap-keypair.json ./target/deploy/openbook_twap.so --final