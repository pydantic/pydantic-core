from pydantic_core import SchemaValidator


def test_model_class():
    v = SchemaValidator(
        {'type': 'model', 'fields': {'field_a': {'schema': {'type': 'str'}}, 'field_b': {'schema': {'type': 'int'}}}}
    )
    # assert repr(v).startswith('SchemaValidator(name="MyModel", validator=NewClass(\n')
    m = v.validate_python({'field_a': 'test', 'field_b': 12})
    assert m.field_a == 'test'
    assert m.field_b == 12
    # m.foobar
    assert m.__fields_set__ == {'field_a', 'field_b'}
    # assert m.__dict__ == {'field_a': 'test', 'field_b': 12}
