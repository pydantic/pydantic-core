from pydantic_core import SchemaValidator, core_schema


def test_nullable_recursive():
    class Child:
        a: int
        b: int

    # Optional[Child]
    v = SchemaValidator(
        core_schema.nullable_schema(
            schema=core_schema.model_schema(
                Child,
                core_schema.model_fields_schema(
                    {
                        'a': core_schema.model_field(core_schema.int_schema()),
                        'b': core_schema.model_field(core_schema.int_schema()),
                    }
                ),
            )
        )
    )
    assert v.construct_python(None) is None
    assert v.construct_python(None, recursive=True) is None
    assert v.construct_python('wrong') == 'wrong'
    assert v.construct_python('wrong', recursive=True) == 'wrong'
    assert v.construct_python({'a': 10, 'b': 'wrong'}) == {'a': 10, 'b': 'wrong'}
    m = v.construct_python({'a': 10, 'b': 'wrong'}, recursive=True)
    assert isinstance(m, Child)
    assert m.a == 10
    assert m.b == 'wrong'
