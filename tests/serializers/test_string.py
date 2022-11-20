import json

import pytest

from pydantic_core import SchemaSerializer, core_schema


def test_str():
    v = SchemaSerializer(core_schema.string_schema())
    assert v.to_python('foobar') == 'foobar'
    assert v.to_python('emoji 💩') == 'emoji 💩'
    assert v.to_json('foobar') == b'"foobar"'
    assert v.to_json('foobar', indent=2) == b'"foobar"'
    assert v.to_json('emoji 💩') == b'"emoji \xf0\x9f\x92\xa9"'
    assert json.loads(v.to_json('emoji 💩')) == 'emoji 💩'

    assert v.to_python('foobar', format='json') == 'foobar'

    # note! serde_json serializes unicode characters differently
    assert v.to_json('emoji 💩') != json.dumps('emoji 💩')


def test_str_fallback():
    s = SchemaSerializer(core_schema.string_schema())
    # we don't (currently) warn on to_python since it uses the default method
    assert s.to_python(123) == 123
    with pytest.warns(UserWarning, match='Expected `str` but got `int` - slight slowdown possible'):
        assert s.to_python(123, format='json') == 123
    with pytest.warns(UserWarning, match='Expected `str` but got `int` - slight slowdown possible'):
        assert s.to_json(123) == b'123'


class Foobar(str):
    pass


@pytest.mark.parametrize('schema_type', ['str', 'any'])
def test_subclass_str(schema_type):
    s = SchemaSerializer({'type': schema_type})
    v = s.to_python(Foobar('foo'))
    assert v == 'foo'
    assert type(v) == Foobar

    v = s.to_python(Foobar('foo'), format='json')
    assert v == 'foo'
    assert type(v) == str

    assert s.to_json(Foobar('foo')) == b'"foo"'
