from pydantic_core import SchemaValidator


def test_generator():
    v = SchemaValidator({'type': 'generator', 'items_schema': {'type': 'int'}})
    s = v.validate_python([1, 2, 3])
    assert list(s) == [1, 2, 3]
