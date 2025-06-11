import pytest

from pydantic_core import PydanticSerializationError, SchemaSerializer, core_schema


def test_to_python_never():
    v = SchemaSerializer(core_schema.never_schema())
    with pytest.raises(TypeError) as exc_info:
        v.to_python(1)
    assert str(exc_info.value) == 'Type `typing.Never` cannot be serialized'


def test_to_json_never():
    v = SchemaSerializer(core_schema.never_schema())
    with pytest.raises(PydanticSerializationError) as exc_info:
        v.to_json('null')
    assert 'Type `typing.Never` cannot be serialized' in str(exc_info.value)
