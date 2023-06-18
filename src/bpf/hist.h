const int tab64[64] = {
    63,  0, 58,  1, 59, 47, 53,  2,
    60, 39, 48, 27, 54, 33, 42,  3,
    61, 51, 37, 40, 49, 18, 28, 20,
    55, 30, 34, 11, 43, 14, 22,  4,
    62, 57, 46, 52, 38, 26, 32, 41,
    50, 36, 17, 19, 29, 10, 13, 21,
    56, 45, 25, 31, 35, 16,  9, 12,
    44, 24, 15,  8, 23,  7,  6,  5};

int log2_(u64 value) // only works for > 0
{
    value |= value >> 1;
    value |= value >> 2;
    value |= value >> 4;
    value |= value >> 8;
    value |= value >> 16;
    value |= value >> 32;
    return tab64[((u64)((value - (value >> 1))*0x07EDD5E59A4E28C2)) >> 58];
}

// inline int log2_(u64 x) {
//      return ((unsigned) (8*sizeof (unsigned long long) - __builtin_clzll((x)) - 1));
// }

/*
  0000 -> (level=0)  0
  1111 -> (level=0) 15 = 0b1111
 10000 -> (level=4) 1000 -> 16 = 0b 10000
 10001 ->                -> 16
 11111 -> (level=4) 1111 -> 16+7 = 23
100000 -> (level=5) 1000 -> 24 = 0b101000
..

4 bits + (level-3)*8 -> less than 500 bins

max = 7 + (63-3)*8 = 7 + 480 = 487 => 488 bins
*/
int hist4_bucket(u64 x) { // 4 significant bits, error ~1/8
    if (x<16) {
        return x;
    } else {
        int log = log2_(x); // log = 4 -> 1
        return ((x >> (log-3)) & (0b1111)) + (log-3)*8;
    }
}
int hist4_offset(int bucket) {
    if (bucket<16) {
        return bucket;
    } else {
        int log = bucket/8+2;
        return ((u64)((bucket & 0b111)+8)) << (log-3);
    }
}
