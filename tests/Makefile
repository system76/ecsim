ASM=$(wildcard *.a51)
C=$(wildcard *.c)
IHX=\
	$(patsubst %.a51,%.tmp/8051.ihx,$(ASM)) \
	$(patsubst %.c,%.tmp/8051.ihx,$(C))
BIN=\
	$(patsubst %.ihx,%.bin,$(IHX))
DISASM=\
	$(patsubst %.ihx,%.a51,$(IHX))

.PHONY: all clean test

all: $(IHX) $(BIN) $(DISASM)

clean:
	rm -rf *.tmp

test: all
	for bin in $(BIN); do \
		dir="$$(dirname "$$bin")" && \
		echo "$${dir%.tmp}" && \
		RUST_BACKTRACE=1 cargo run \
		 	--quiet \
			--manifest-path ../Cargo.toml \
			--no-default-features \
			-- $$bin | \
		tee "$${bin%.bin}.stdout" ; \
	done

%.tmp/8051.ihx: %.a51
	rm -rf $*.tmp
	mkdir -p $*.tmp
	cd $*.tmp && \
	as31 -O8051.ihx ../$<

%.tmp/8051.ihx: %.c
	rm -rf $*.tmp
	mkdir -p $*.tmp
	cd $*.tmp && \
	sdcc -mmcs51 -o 8051.ihx ../$<

%.tmp/8051.bin: %.tmp/8051.ihx
	makebin -p < $< > $@

%.tmp/8051.a51: %.tmp/8051.ihx
	dis51 < $< > $@
