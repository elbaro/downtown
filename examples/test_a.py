from test_b import A

a = A()


def h():
    a.f()
    a.g()


while True:
    h()
