bool streqn(const char* a, const char *b, int n) {
    while(n--) {
        if (*a != *b) return false;
        if (*a==0) break;
        ++a;
        ++b;
    }
    return true;
}
