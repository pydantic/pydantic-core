import pytest

from pydantic_core import SchemaSerializer, core_schema


def gen_ok(*things):
    for thing in things:
        yield thing


def gen_error(*things):
    for thing in things:
        yield thing
    raise ValueError('oops')


def test_generator_any():
    s = SchemaSerializer(core_schema.generator_schema(core_schema.any_schema()))
    assert s.to_python(iter(['a', b'b', 3])) == ['a', b'b', 3]
    assert s.to_python(gen_ok('a', b'b', 3)) == ['a', b'b', 3]
    assert s.to_python(('a', b'b', 3)) == ['a', b'b', 3]
    assert s.to_python('abc') == ['a', 'b', 'c']

    assert s.to_python(iter(['a', b'b', 3]), mode='json') == ['a', 'b', 3]

    assert s.to_json(iter(['a', b'b', 3])) == b'["a","b",3]'
    assert s.to_json(gen_ok('a', b'b', 3)) == b'["a","b",3]'
    assert s.to_json(('a', b'b', 3)) == b'["a","b",3]'

    with pytest.warns(UserWarning, match='Expected `generator` but got `int` - filtering via include/exclude'):
        assert s.to_python(4) == 4

    with pytest.raises(ValueError, match='oops'):
        s.to_python(gen_error(1, 2))


def test_generator_int():
    s = SchemaSerializer(core_schema.generator_schema(core_schema.int_schema()))
    assert s.to_python(iter([1, 2, 3])) == [1, 2, 3]
    assert s.to_python(gen_ok(1, 2, 3)) == [1, 2, 3]
    assert s.to_python((1, 2, 3)) == [1, 2, 3]

    assert s.to_python(iter([1, 2, 3]), mode='json') == [1, 2, 3]

    assert s.to_json(iter([1, 2, 3])) == b'[1,2,3]'
    assert s.to_json(gen_ok(1, 2, 3)) == b'[1,2,3]'
    assert s.to_json((1, 2, 3)) == b'[1,2,3]'

    with pytest.raises(ValueError, match='oops'):
        s.to_python(gen_error(1, 2))
