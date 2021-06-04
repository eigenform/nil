#ifdef NATIVE
#include <stdio.h>
#endif

static unsigned int buffer[32] = {
	0x00000001, 0x00000010, 0x00000100, 0x00001000, 
	0x00000002, 0x00000020, 0x00000200, 0x00002000,
	0x00000004, 0x00000040, 0x00000400, 0x00004000,
	0x00000008, 0x00000080, 0x00000800, 0x00008000,
	0x00010000, 0x00100000, 0x01000000, 0x10000000, 
	0x00020000, 0x00200000, 0x02000000, 0x20000000, 
	0x00040000, 0x00400000, 0x04000000, 0x40000000, 
	0x00080000, 0x00800000, 0x08000000, 0x80000000, 
};


extern unsigned int add_array(unsigned int tmp);
extern unsigned int sub_array(unsigned int tmp);
extern unsigned int and_not_array(unsigned int tmp);
extern unsigned int or_array(unsigned int tmp);

int main() {
	int ret = 0;
	unsigned int res[16];

	res[0] = add_array(0x00000000);
	res[1] = sub_array(0xffffffff);
	res[2] = and_not_array(0xffffffff);
	res[3] = or_array(0x00000000);

#ifdef NATIVE
	printf("add_array=%08x\n", res[0]);
	printf("sub_array=%08x\n", res[1]);
	printf("and_not_array=%08x\n", res[2]);
	printf("or_array=%08x\n", res[3]);
#endif

	return ret;
}

unsigned int add_array(unsigned int tmp) {
	for (int i = 0; i < 32; i++) { tmp += buffer[i]; }
	return tmp;
}
unsigned int sub_array(unsigned int tmp) {
	for (int i = 0; i < 32; i++) { tmp -= buffer[i]; }
	return tmp;
}
unsigned int and_not_array(unsigned int tmp) {
	for (int i = 0; i < 32; i++) { tmp &= ~buffer[i]; }
	return tmp;
}
unsigned int or_array(unsigned int tmp) {
	for (int i = 0; i < 32; i++) { tmp |= buffer[i]; }
	return tmp;
}



