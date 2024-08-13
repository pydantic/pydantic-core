import pytest
from dirty_equals import IsStr

from pydantic_core import SchemaSerializer, core_schema
import pydantic_core


def gen_ok(*things):
    yield from things


def gen_error(*things):
    yield from things
    raise ValueError('oops')


def test_generator_any_iter():
    s = SchemaSerializer(core_schema.generator_schema(core_schema.any_schema()))
    gen = s.to_python(gen_ok('a', b'b', 3))
    assert repr(gen) == IsStr(regex=r'SerializationIterator\(index=0, iterator=<generator object gen_ok at 0x\w+>\)')
    assert str(gen) == repr(gen)
    assert gen.index == 0
    assert next(gen) == 'a'
    assert gen.index == 1
    assert repr(gen) == IsStr(regex=r'SerializationIterator\(index=1, iterator=<generator object gen_ok at 0x\w+>\)')
    assert next(gen) == b'b'
    assert gen.index == 2
    assert next(gen) == 3
    assert gen.index == 3
    with pytest.raises(StopIteration):
        next(gen)
    assert gen.index == 3


def test_any_iter():
    s = SchemaSerializer(core_schema.any_schema())
    gen = s.to_python(gen_ok('a', b'b', 3))
    assert repr(gen) == IsStr(regex=r'SerializationIterator\(index=0, iterator=<generator object gen_ok at 0x\w+>\)')
    assert str(gen) == repr(gen)
    assert next(gen) == 'a'
    assert repr(gen) == IsStr(regex=r'SerializationIterator\(index=1, iterator=<generator object gen_ok at 0x\w+>\)')
    assert next(gen) == b'b'
    assert next(gen) == 3
    with pytest.raises(StopIteration):
        next(gen)


def test_generator_any():
    s = SchemaSerializer(core_schema.generator_schema(core_schema.any_schema()))
    assert list(s.to_python(iter(['a', b'b', 3]))) == ['a', b'b', 3]
    assert list(s.to_python(gen_ok('a', b'b', 3))) == ['a', b'b', 3]

    assert s.to_python(iter(['a', b'b', 3]), mode='json') == ['a', 'b', 3]

    assert s.to_json(iter(['a', b'b', 3])) == b'["a","b",3]'
    assert s.to_json(gen_ok('a', b'b', 3)) == b'["a","b",3]'

    msg = 'Expected `generator` but got `int` with value `4` - serialized value may not be as expected'
    with pytest.warns(UserWarning, match=msg):
        assert s.to_python(4) == 4
    with pytest.warns(UserWarning, match="Expected `generator` but got `tuple` with value `\\('a', b'b', 3\\)`"):
        assert s.to_python(('a', b'b', 3)) == ('a', b'b', 3)
    with pytest.warns(UserWarning, match="Expected `generator` but got `str` with value `'abc'`"):
        assert s.to_python('abc') == 'abc'

    with pytest.raises(ValueError, match='oops'):
        list(s.to_python(gen_error(1, 2)))

    with pytest.raises(ValueError, match='oops'):
        s.to_python(gen_error(1, 2), mode='json')

    with pytest.raises(ValueError, match='oops'):
        s.to_json(gen_error(1, 2))


def test_generator_int():
    s = SchemaSerializer(core_schema.generator_schema(core_schema.int_schema()))
    assert list(s.to_python(iter([1, 2, 3]))) == [1, 2, 3]
    assert list(s.to_python(gen_ok(1, 2, 3))) == [1, 2, 3]

    assert s.to_python(iter([1, 2, 3]), mode='json') == [1, 2, 3]

    assert s.to_json(iter([1, 2, 3])) == b'[1,2,3]'
    assert s.to_json(gen_ok(1, 2, 3)) == b'[1,2,3]'

    with pytest.raises(ValueError, match='oops'):
        list(s.to_python(gen_error(1, 2)))

    with pytest.raises(ValueError, match='oops'):
        s.to_json(gen_error(1, 2))

    with pytest.warns(
        UserWarning, match="Expected `int` but got `str` with value `'a'` - serialized value may not be as expected"
    ):
        s.to_json(gen_ok(1, 'a'))

    gen = s.to_python(gen_ok(1, 'a'))
    assert next(gen) == 1
    with pytest.warns(
        UserWarning, match="Expected `int` but got `str` with value `'a'` - serialized value may not be as expected"
    ):
        assert next(gen) == 'a'
    with pytest.warns(
        UserWarning,
        match='Expected `generator` but got `tuple` with value `\\(1, 2, 3\\)` - serialized value may not.+',
    ):
        s.to_python((1, 2, 3))


def test_include():
    v = SchemaSerializer(
        core_schema.generator_schema(
            core_schema.any_schema(), serialization=core_schema.filter_seq_schema(include={1, 3, 5})
        )
    )
    assert v.to_python(gen_ok(0, 1, 2, 3), mode='json') == [1, 3]
    assert list(v.to_python(gen_ok(0, 1, 2, 3))) == [1, 3]
    assert v.to_python(gen_ok('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'), mode='json') == ['b', 'd', 'f']
    assert v.to_python(gen_ok('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'), mode='json') == ['b', 'd', 'f']
    assert v.to_json(gen_ok('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h')) == b'["b","d","f"]'
    # the two include lists are now combined via UNION! unlike in pydantic v1
    assert v.to_python(gen_ok('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'), include={6}, mode='json') == ['b', 'd', 'f', 'g']
    assert list(v.to_python(gen_ok('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'), include={6})) == ['b', 'd', 'f', 'g']
    assert v.to_json(gen_ok('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'), include={6}) == b'["b","d","f","g"]'
    assert v.to_python(gen_ok('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'), include={6: None}, mode='json') == [
        'b',
        'd',
        'f',
        'g',
    ]
    with pytest.raises(ValueError, match='Negative indices cannot be used to exclude items on unsized iterables'):
        v.to_python(gen_ok('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'), include={-1: None, -2: None}, mode='json')
    # Non-integer keys are ignored
    v.to_python(gen_ok('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'), include={'__all__': None}, mode='json')


def test_custom_serializer():
    s = SchemaSerializer(core_schema.any_schema(serialization=core_schema.simple_ser_schema('generator')))
    assert s.to_python(gen_ok(1, 2), mode='json') == [1, 2]
    assert s.to_json(gen_ok(1, 2)) == b'[1,2]'


def test_generator_no_ser_any_iter():
    s = SchemaSerializer(core_schema.generator_schema(core_schema.any_schema()))
    gen_inner = gen_ok('a', b'b', 3)
    gen = s.to_python(gen_inner, serialize_generators=False)
    assert gen_inner is gen


def test_any_no_ser_iter():
    s = SchemaSerializer(core_schema.any_schema())
    gen_inner = gen_ok('a', b'b', 3)
    gen = s.to_python(gen_inner, serialize_generators=False)
    assert gen is gen_inner

    with pytest.raises(ValueError, match='Unable to serialize unknown type'):
        assert s.to_json(iter(['a', b'b', 3]), serialize_generators=False) == b'["a","b",3]'


def test_generator_no_ser_any():
    # todo
    s = SchemaSerializer(core_schema.generator_schema(core_schema.any_schema()))
    assert list(s.to_python(iter(['a', b'b', 3]), serialize_generators=False)) == ['a', b'b', 3]
    assert list(s.to_python(gen_ok('a', b'b', 3), serialize_generators=False)) == ['a', b'b', 3]

    with pytest.raises(ValueError, match='Unable to serialize unknown type'):
        assert s.to_python(iter(['a', b'b', 3]), mode='json', serialize_generators=False) == ['a', 'b', 3]

    with pytest.raises(ValueError, match='Unable to serialize unknown type'):
        assert s.to_json(iter(['a', b'b', 3]), serialize_generators=False) == b'["a","b",3]'
    assert s.to_json(iter(['a', b'b', 3]), serialize_generators=False, fallback=list) == b'["a","b",3]'
    assert s.to_json(gen_ok('a', b'b', 3), serialize_generators=False, fallback=list) == b'["a","b",3]'
    assert s.to_python(gen_ok('a', b'b', 3), serialize_generators=False, fallback=list) == ["a", b"b", 3]

    msg = 'Expected `generator` but got `int` with value `4` - serialized value may not be as expected'
    with pytest.warns(UserWarning, match=msg):
        assert s.to_python(4, serialize_generators=False) == 4
    with pytest.warns(UserWarning, match="Expected `generator` but got `tuple` with value `\\('a', b'b', 3\\)`"):
        assert s.to_python(('a', b'b', 3), serialize_generators=False) == ('a', b'b', 3)
    with pytest.warns(UserWarning, match="Expected `generator` but got `str` with value `'abc'`"):
        assert s.to_python('abc', serialize_generators=False) == 'abc'

    with pytest.raises(ValueError, match='oops'):
        list(s.to_python(gen_error(1, 2), serialize_generators=False))

    with pytest.raises(ValueError, match='Unable to serialize unknown type'):
        s.to_python(gen_error(1, 2), mode='json', serialize_generators=False)

    with pytest.raises(ValueError, match='Unable to serialize unknown type'):
        s.to_json(gen_error(1, 2), serialize_generators=False)


def test_no_ser_custom_serializer():
    s = SchemaSerializer(core_schema.any_schema(serialization=core_schema.simple_ser_schema('generator')))
    with pytest.raises(ValueError, match='Unable to serialize unknown type'):
        assert s.to_python(gen_ok(1, 2), mode='json', serialize_generators=False) == [1, 2]
    with pytest.raises(ValueError, match='Unable to serialize unknown type'):
        assert s.to_json(gen_ok(1, 2), serialize_generators=False) == b'[1,2]'
    assert s.to_python(gen_ok(1, 2), mode='json', serialize_generators=False, fallback=list) == [1, 2]
    assert s.to_json(gen_ok(1, 2), serialize_generators=False, fallback=list) == b'[1,2]'

def test_no_ser_to_json():
    with pytest.raises(ValueError, match='Unable to serialize unknown type'):
        assert pydantic_core.to_jsonable_python(iter([]), serialize_generators=False)
    assert pydantic_core.to_jsonable_python(iter([]), serialize_generators=False, serialize_unknown=True) == IsStr(regex=r'<list_iterator object at 0x\w+>')
    assert pydantic_core.to_json(iter([]), serialize_generators=False, serialize_unknown=True).decode('utf8') == IsStr(regex=r'"<list_iterator object at 0x\w+>"')
    assert pydantic_core.to_json(iter([]), serialize_generators=False, serialize_unknown=True).decode('utf8') == IsStr(regex=r'"<list_iterator object at 0x\w+>"')
