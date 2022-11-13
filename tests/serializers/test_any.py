import json

import pytest

from pydantic_core import SchemaSerializer, core_schema

from ..conftest import plain_repr


@pytest.fixture(scope='module')
def any_serializer():
    return SchemaSerializer(core_schema.any_schema())


def test_repr(any_serializer):
    assert plain_repr(any_serializer) == 'SchemaSerializer(serializer=Any(AnySerializer))'


@pytest.mark.parametrize('value', [None, 1, 1.0, True, 'foo', [1, 2, 3], {'a': 1, 'b': 2}])
def test_any_json_round_trip(any_serializer, value):
    assert any_serializer.to_python(value) == value
    assert json.loads(any_serializer.to_json(value)) == value
    assert any_serializer.to_python(value, format='json') == value


@pytest.mark.parametrize(
    'value,expected_json',
    [
        (None, b'null'),
        (1, b'1'),
        (b'foobar', b'"foobar"'),
        (bytearray(b'foobar'), b'"foobar"'),
        ((1, 2, 3), b'[1,2,3]'),
    ],
)
def test_any_json_coerce(any_serializer, value, expected_json):
    assert any_serializer.to_python(value) == value
    assert any_serializer.to_json(value) == expected_json
    assert any_serializer.to_python(value, format='json') == json.loads(expected_json)


def test_other_type():
    """Types with no serializer, fall back to any serializer"""
    v = SchemaSerializer(core_schema.is_instance_schema(int))
    assert plain_repr(v) == 'SchemaSerializer(serializer=Any(AnySerializer))'
    assert v.to_json('foobar') == b'"foobar"'
