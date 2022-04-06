import timeit
from decimal import Decimal
from enum import Enum
from statistics import mean

from devtools import debug

import plain


def benchmark_str_validation():
    from pydantic_core import _pydantic_core as rust

    impls = plain, rust

    class Foo(str, Enum):
        bar = 'bar'
        baz = 'baz'
        qux = 'qux'

    choices = [
        'this is a string',
        'this is another string',
        'this is a third string',
        b'hello ',
        Foo.bar,
        123,
        123.456,
        Decimal('321.123'),
        [1, 2, 3,  'this is a string', b'hello ', Foo.bar, 123, 123.456, Decimal('321.123')],
        {'a': 'this is a string', 'b': 123, 'c': Foo.baz},
        # object(),
    ]

    data = {
        'str': 'this is a string',
        'list': choices,
        'dict': {'foo': 'bar', 'baz': choices},
    }

    old_result = None
    steps = 1_000

    for impl in impls:
        print(f'{impl.__name__} validate_str_recursive:')
        result = impl.validate_str_recursive(data, None, 50, True, False, True)
        # debug(result)
        if old_result:
            assert result == old_result
        old_result = result

        big_data = [data] * 100
        t = timeit.timeit(
            'impl.validate_str_recursive(big_data, None, 50, True, False, True)',
            globals={'impl': impl, 'big_data': big_data},
            number=steps,
        )
        print(f'    {t / steps * 1_000_000:.2f}µs\n')


def benchmark_regex():
    from pydantic_core._pydantic_core import PyRegex, RustRegex, OnigRegex

    impls = PyRegex, RustRegex, OnigRegex
    steps = 100_000

    for impl in impls:
        print(f'{impl.__name__}:')
        # https://stackoverflow.com/a/3809435/949890
        r = impl(
            r'https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)'
        )
        assert r.test('https://www.google.com')
        assert not r.test('foobar')

        times = (
            timeit.timeit('r.test("https://www.google.com")', globals={'r': r}, number=steps),
            timeit.timeit('r.test("foobar")', globals={'r': r}, number=steps),
        )
        print(f'    {mean(times) / steps * 1_000_000:.2f}µs\n')


if __name__ == '__main__':
    benchmark_str_validation()
    benchmark_regex()
