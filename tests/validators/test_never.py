import pytest

from pydantic_core import PydanticUndefined, SchemaValidator, ValidationError, core_schema


def test_python_never():
    v = SchemaValidator(core_schema.never_schema())
    with pytest.raises(ValidationError) as exc_info:
        v.validate_python(1)
    assert exc_info.value.errors(include_url=False) == [
        {'type': 'never', 'loc': (), 'msg': 'No input is allowed for `typing.Never`', 'input': 1}
    ]

    assert v.validate_python(PydanticUndefined) is PydanticUndefined


def test_json_never():
    v = SchemaValidator(core_schema.never_schema())
    with pytest.raises(ValidationError) as exc_info:
        v.validate_json('null')
    assert exc_info.value.errors(include_url=False) == [
        {'type': 'never', 'loc': (), 'msg': 'No input is allowed for `typing.Never`', 'input': None}
    ]

    class MyModel:
        pass

    schema = core_schema.model_schema(
        MyModel,
        core_schema.model_fields_schema(
            {
                'a': core_schema.model_field(core_schema.never_schema()),
            }
        ),
    )
    v = SchemaValidator(schema)
    m = v.validate_json('{}')
    assert m.a is PydanticUndefined
