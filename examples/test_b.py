class A:
    @aa
    def f(self):
        a = 1
        for i in range(100):
            a = a * 2

    def g(self):
        a = 1
        b = 4
        for i in range(100):
            a = a * 2 - b
            b = b * -2 + a
