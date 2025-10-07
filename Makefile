all:
	cargo run -- submit --order-type=sell --product-id=7 --price=1 --quantity=1000000 --price-type=limit
sell:
	cargo run -- submit --order-type=sell --product-id=7 --price=1 --quantity=1 --price-type=limit
buy:
	cargo run -- submit --order-type=buy --product-id=7 --price=20000 --quantity=1 --price-type=limit
