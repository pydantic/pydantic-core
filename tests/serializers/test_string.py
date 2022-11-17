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
    v = SchemaSerializer(core_schema.string_schema())
    # we don't (currently) warn on to_python since it uses the default method
    assert v.to_python(123) == 123
    with pytest.warns(UserWarning, match='Expected `str` but got `int`'):
        assert v.to_json(123) == b'123'
