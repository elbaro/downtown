import asyncio


async def pause():
    await asyncio.sleep(1)


async def myfunc_f2():
    return [await pause(), await pause()]


def k():
    pass


def kk():
    raise RuntimeError("test")


async def myfunc_g2():
    k()
    kk()
    k()


async def myfunc_f():
    a = 3
    await pause()
    d = 6
    await pause()


async def myfunc_g():
    await pause()
    b = 5
    await pause()
    c = 7


async def main():
    await myfunc_g2()
    # await asyncio.gather(myfunc_f(), myfunc_g())


asyncio.run(main())
