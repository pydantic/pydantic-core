import json
from datetime import date, datetime, time, timedelta

import pytest

from pydantic_core import PydanticSerializationError, SchemaSerializer, core_schema

from ..conftest import plain_repr
from .test_list_tuple import as_list, as_tuple


@pytest.fixture(scope='module')
def any_serializer():
    return SchemaSerializer(core_schema.any_schema())


def test_repr(any_serializer):
    assert plain_repr(any_serializer) == 'SchemaSerializer(serializer=Any(AnySerializer),slots=[])'


@pytest.mark.parametrize('value', [None, 1, 1.0, True, 'foo', [1, 2, 3], {'a': 1, 'b': 2}])
def test_any_json_round_trip(any_serializer, value):
    assert any_serializer.to_python(value) == value
    assert json.loads(any_serializer.to_json(value)) == value
    assert any_serializer.to_python(value, mode='json') == value


@pytest.mark.parametrize(
    'value,expected_json',
    [
        (None, b'null'),
        (1, b'1'),
        (b'foobar', b'"foobar"'),
        (bytearray(b'foobar'), b'"foobar"'),
        ((1, 2, 3), b'[1,2,3]'),
        (datetime(2022, 12, 3, 12, 30, 45), b'"2022-12-03T12:30:45"'),
        (date(2022, 12, 3), b'"2022-12-03"'),
        (time(12, 30, 45), b'"12:30:45"'),
        (timedelta(hours=2), b'"PT7200S"'),
    ],
)
def test_any_json_coerce(any_serializer, value, expected_json):
    assert any_serializer.to_python(value) == value
    assert any_serializer.to_json(value) == expected_json
    assert any_serializer.to_python(value, mode='json') == json.loads(expected_json)


def test_other_type():
    """Types with no serializer, fall back to any serializer"""
    v = SchemaSerializer(core_schema.is_instance_schema(int))
    assert plain_repr(v) == 'SchemaSerializer(serializer=Any(AnySerializer),slots=[])'
    assert v.to_json('foobar') == b'"foobar"'


@pytest.mark.parametrize('value', [b'\x81', bytearray(b'\x81')])
def test_any_json_decode_error(any_serializer, value):
    assert any_serializer.to_python(value) == value

    msg = 'Error serializing to JSON: invalid utf-8 sequence of 1 bytes from index 0'
    with pytest.raises(PydanticSerializationError, match=msg):
        any_serializer.to_json(value)

    with pytest.raises(ValueError):
        any_serializer.to_python(value, mode='json')


def test_any_with_date_serializer():
    s = SchemaSerializer(core_schema.any_schema(serialization={'type': 'date'}))
    assert s.to_python(date(2022, 12, 3)) == date(2022, 12, 3)
    assert s.to_python(date(2022, 12, 3), mode='json') == '2022-12-03'
    assert s.to_json(date(2022, 12, 3)) == b'"2022-12-03"'

    with pytest.warns(UserWarning) as warning_info:
        assert s.to_python(b'bang', mode='json') == 'bang'

    assert [w.message.args[0] for w in warning_info.list] == [
        'Pydantic serializer warnings:\n  Expected `date` but got `bytes` - slight slowdown possible'
    ]


def test_any_with_timedelta_serializer():
    s = SchemaSerializer(core_schema.any_schema(serialization={'type': 'timedelta'}))
    assert s.to_python(timedelta(hours=2)) == timedelta(hours=2)
    assert s.to_python(timedelta(hours=2), mode='json') == 'PT7200S'
    assert s.to_json(timedelta(hours=2)) == b'"PT7200S"'

    with pytest.warns(UserWarning) as warning_info:
        assert s.to_python(b'bang', mode='json') == 'bang'

    assert [w.message.args[0] for w in warning_info.list] == [
        'Pydantic serializer warnings:\n  Expected `timedelta` but got `bytes` - slight slowdown possible'
    ]


def test_any_config_timedelta_float():
    s = SchemaSerializer(core_schema.any_schema(), config={'ser_json_timedelta': 'float'})
    h2 = timedelta(hours=2)
    assert s.to_python(h2) == h2
    assert s.to_python(h2, mode='json') == 7200.0
    assert s.to_json(h2) == b'7200.0'

    assert s.to_python({h2: 'foo'}) == {h2: 'foo'}
    assert s.to_python({h2: 'foo'}, mode='json') == {'7200': 'foo'}
    assert s.to_json({h2: 'foo'}) == b'{"7200":"foo"}'


def test_any_config_timedelta_float_faction():
    s = SchemaSerializer(core_schema.any_schema(), config={'ser_json_timedelta': 'float'})
    one_half_s = timedelta(seconds=1.5)
    assert s.to_python(one_half_s) == one_half_s
    assert s.to_python(one_half_s, mode='json') == 1.5
    assert s.to_json(one_half_s) == b'1.5'

    assert s.to_python({one_half_s: 'foo'}) == {one_half_s: 'foo'}
    assert s.to_python({one_half_s: 'foo'}, mode='json') == {'1.5': 'foo'}
    assert s.to_json({one_half_s: 'foo'}) == b'{"1.5":"foo"}'


def test_recursion():
    s = SchemaSerializer(core_schema.any_schema())
    v = [1, 2]
    v.append(v)
    assert s.to_python(v) == v
    with pytest.raises(ValueError, match=r'Circular reference detected \(id repeated\)'):
        s.to_python(v, mode='json')
    with pytest.raises(ValueError, match=r'Circular reference detected \(id repeated\)'):
        s.to_json(v)


@pytest.mark.parametrize('seq_f', [as_list, as_tuple])
def test_include_list_tuple(seq_f):
    v = SchemaSerializer(core_schema.any_schema())
    assert v.to_python(seq_f(0, 1, 2, 3)) == seq_f(0, 1, 2, 3)
    assert v.to_python(seq_f('a', 'b', 'c')) == seq_f('a', 'b', 'c')
    assert v.to_python(seq_f('a', 'b', 'c'), mode='json') == ['a', 'b', 'c']
    assert v.to_json(seq_f('a', 'b', 'c')) == b'["a","b","c"]'

    assert v.to_python(seq_f(0, 1, 2, 3), include={1, 2}) == seq_f(1, 2)
    assert v.to_python(seq_f(0, 1, 2, 3), include={1, 2}, mode='json') == [1, 2]
    assert v.to_python(seq_f('a', 'b', 'c', 'd'), include={1, 2}) == seq_f('b', 'c')
    assert v.to_python(seq_f('a', 'b', 'c', 'd'), include={1, 2}, mode='json') == ['b', 'c']
    assert v.to_json(seq_f('a', 'b', 'c', 'd'), include={1, 2}) == b'["b","c"]'


def test_include_dict():
    v = SchemaSerializer(core_schema.any_schema())
    assert v.to_python({1: 2, '3': 4}) == {1: 2, '3': 4}
    assert v.to_python({1: 2, '3': 4}, mode='json') == {'1': 2, '3': 4}
    assert v.to_json({1: 2, '3': 4}) == b'{"1":2,"3":4}'

    assert v.to_python({1: 2, '3': 4}, include={1}) == {1: 2}
    assert v.to_python({1: 2, '3': 4}, include={'3'}) == {'3': 4}
    assert v.to_python({1: 2, '3': 4}, include={1}, mode='json') == {'1': 2}
    assert v.to_python({1: 2, '3': 4}, include={'3'}, mode='json') == {'3': 4}
    assert v.to_json({1: 2, '3': 4}, include={1}) == b'{"1":2}'
    assert v.to_json({1: 2, '3': 4}, include={'3'}) == b'{"3":4}'
