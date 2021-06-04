
# Note that I have `armebv5te-none-eabi-{gcc,ld}` symlinked to my copies of
# `arm-none-eabi-{gcc,ld}` which are in my $PATH; I think that Clang uses the
# current target to decide what to call when it's time to do linking.
# Ideally I'd just use `lld`, but it seems like armeb is somewhat broken?

SYSROOT := /opt/devkitpro/devkitARM/
TRIPLE 	:= armebv5te-none-eabi
ARCH 	:= armv5te
CPU 	:= arm926ej-s

LDF 	:= harness.ldf
LDFLAGS := -Wl,-EB,--script=$(LDF),-Map,test.map

SRC 	:= test.c

.PHONY: all ir clean dis
all:
	clang -DNATIVE $(SRC) -o test.x64.elf
	clang --target=$(TRIPLE) -march=$(ARCH) -mcpu=$(CPU) \
		--sysroot=$(SYSROOT) $(LDFLAGS) -nostdlib $(SRC) -o test.arm.elf
	arm-none-eabi-objcopy -O binary test.arm.elf test.bin
	clang --target=$(TRIPLE) -march=$(ARCH) -mcpu=$(CPU) \
		--sysroot=$(SYSROOT) -emit-llvm -S $(SRC)

clean:
	rm -fv *.ll *.o *.map *.elf *.bin a.out
dis:
	arm-none-eabi-objdump -d test.arm.elf

