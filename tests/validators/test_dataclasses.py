from pydantic_core import SchemaValidator, core_schema


def test_dataclass():
    schema = core_schema.dataclass_schema(
        core_schema.dataclass_field(name='a', schema=core_schema.str_schema(), positional=True),
        core_schema.dataclass_field(name='b', schema=core_schema.bool_schema(), positional=True),
    )
    v = SchemaValidator(schema)
    assert v.validate_python(('hello', True)) == {'a': 'hello', 'b': True}
