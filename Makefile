all:
	#cargo run -- submit --order-type=sell --product-id=7 --price=1 --quantity=10 --price-type=limit
	cargo run -- submit --order-type=mocksell --product-id=7 --price=1 --quantity=10 --price-type=limit
	
sell:
	cargo run -- submit --order-type=sell --product-id=7 --price=1 --quantity=10000 --price-type=limit
buy:
	cargo run -- submit --order-type=buy --product-id=7 --price=20000 --quantity=1 --price-type=limit
