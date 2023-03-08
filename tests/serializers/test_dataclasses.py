from pydantic_core import SchemaSerializer, core_schema


def test_dataclass():
    schema = core_schema.dataclass_args_schema(
        core_schema.dataclass_field(name='a', schema=core_schema.str_schema(), positional=True),
        core_schema.dataclass_field(name='b', schema=core_schema.bool_schema(), positional=True),
    )
    s = SchemaSerializer(schema)
    assert s.to_python({'a': 'hello', 'b': True}) == {'a': 'hello', 'b': True}
    assert s.to_python({'a': 'hello', 'b': True}, exclude={'a'}) == {'b': True}
