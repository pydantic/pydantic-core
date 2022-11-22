import json
from enum import Enum

import pytest

from pydantic_core import PydanticSerializationError, SchemaSerializer, core_schema


def test_bytes():
    s = SchemaSerializer(core_schema.bytes_schema())
    assert s.to_python(b'foobar') == b'foobar'
    assert s.to_python('emoji 💩'.encode()) == 'emoji 💩'.encode()
    assert s.to_json(b'foobar') == b'"foobar"'
    assert s.to_python(b'foobar', mode='json') == 'foobar'

    json_emoji = s.to_json('emoji 💩'.encode())
    # note! serde_json serializes unicode characters differently
    assert json_emoji == b'"emoji \xf0\x9f\x92\xa9"'
    assert json.loads(json_emoji) == 'emoji 💩'


def test_bytes_invalid():
    s = SchemaSerializer(core_schema.bytes_schema())
    assert s.to_python(b'\x81') == b'\x81'

    with pytest.raises(UnicodeDecodeError, match="'utf-8' codec can't decode byte 0x81 in position 0: invalid utf-8"):
        s.to_python(b'\x81', mode='json')

    msg = 'Error serializing to JSON: invalid utf-8 sequence of 1 bytes from index 0'
    with pytest.raises(PydanticSerializationError, match=msg):
        s.to_json(b'\x81')


def test_bytes_dict_key():
    s = SchemaSerializer(core_schema.dict_schema(core_schema.bytes_schema(), core_schema.int_schema()))
    assert s.to_python({b'foobar': 123}) == {b'foobar': 123}
    assert s.to_python({b'foobar': 123}, mode='json') == {'foobar': 123}
    assert s.to_json({b'foobar': 123}) == b'{"foobar":123}'


def test_bytes_fallback():
    s = SchemaSerializer(core_schema.bytes_schema())
    # we don't (currently) warn on to_python since it uses the default method
    assert s.to_python(123) == 123
    with pytest.warns(UserWarning, match='Expected `bytes` but got `int` - slight slowdown possible'):
        assert s.to_python(123, mode='json') == 123
    with pytest.warns(UserWarning, match='Expected `bytes` but got `int` - slight slowdown possible'):
        assert s.to_json(123) == b'123'
    with pytest.warns(UserWarning, match='Expected `bytes` but got `str` - slight slowdown possible'):
        assert s.to_json('foo') == b'"foo"'


class BytesSubclass(bytes):
    pass


class BasicClass:
    pass


class BytesMixin(bytes, BasicClass):
    pass


class BytesEnum(bytes, Enum):
    foo = b'foo-value'
    bar = b'bar-value'


@pytest.mark.parametrize('schema_type', ['bytes', 'any'])
@pytest.mark.parametrize(
    'input_value,expected_json',
    [(BytesSubclass(b'foo'), 'foo'), (BytesMixin(b'foo'), 'foo'), (BytesEnum.foo, 'foo-value')],
)
def test_subclass_bytes(schema_type, input_value, expected_json):
    s = SchemaSerializer({'type': schema_type})
    v = s.to_python(input_value)
    assert v == input_value
    assert type(v) == type(input_value)

    v = s.to_python(input_value, mode='json')
    assert v == expected_json
    assert type(v) == str

    assert s.to_json(input_value) == json.dumps(expected_json).encode('utf-8')
