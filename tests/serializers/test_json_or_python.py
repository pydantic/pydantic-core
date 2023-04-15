import pytest

from pydantic_core import SchemaSerializer, core_schema

from ..conftest import plain_repr


def test_lax_or_strict():
    s = SchemaSerializer(core_schema.json_or_python_schema(core_schema.float_schema(), core_schema.int_schema()))

    assert s.to_json(123) == b'123.0'
    with pytest.warns(UserWarning, match='Expected `float` but got `str` - serialized value may not be as expected'):
        assert s.to_json('123') == b'"123"'
    assert s.to_python(123) == 123 and isinstance(s.to_python(123), int)

    # insert_assert(plain_repr(s))
    assert plain_repr(s) == (
        'SchemaSerializer(serializer=JsonOrPython(JsonOrPythonSerializer{json:Float(FloatSerializer),python:Int(IntSerializer),name:"json-or-python[json=float,python=int]"}),slots=[])'
    )
