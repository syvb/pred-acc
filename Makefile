all: data
.PHONY: all dl data

dl:
	$(MAKE) -C data dl

data:
	$(MAKE) -C data
