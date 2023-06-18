// https://stackoverflow.com/questions/2351087/what-is-the-best-32bit-hash-function-for-short-strings-tag-names
// > Empirically, the values 31 and 37 have proven to be good choices for the multiplier in a hash function for ASCII strings.

const u64 MULTIPLIER = 37;

u64 hash(char *str)
{
   u64 h = 0;
   unsigned char *p;

   for (p = (unsigned char*)str; *p != '\0'; p++)
      h = MULTIPLIER * h + *p;

   return h; // or, h % ARRAY_SIZE;
}
