from pydantic_core import SchemaSerializer, core_schema


def test_list_any():
    v = SchemaSerializer(core_schema.list_schema(core_schema.any_schema()))
    assert v.to_python(['a', 'b', 'c']) == ['a', 'b', 'c']
    assert v.to_python(['a', 'b', 'c'], format='json') == ['a', 'b', 'c']
    assert v.to_json(['a', 'b', 'c']) == b'["a","b","c"]'

    assert v.to_json(['a', 'b', 'c'], indent=2) == b'[\n  "a",\n  "b",\n  "c"\n]'


def test_list_callback():
    v = SchemaSerializer(core_schema.list_schema(core_schema.any_schema()))
    assert v.to_python('apple') == 'apple'
    assert v.to_json('apple') == b'"apple"'
    assert v.to_json(b'apple') == b'"apple"'
    assert v.to_python((1, 2, 3)) == (1, 2, 3)
    # even though we're in the fallback state, non JSON types should still be converted to JSON here
    assert v.to_python((1, 2, 3), format='json') == [1, 2, 3]


def test_build_time_include():
    v = SchemaSerializer(core_schema.list_schema(core_schema.any_schema(), serialization={'include': {1, 3, 5}}))
    assert v.to_python(['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h']) == ['b', 'd', 'f']


def test_build_time_exclude():
    v = SchemaSerializer(core_schema.list_schema(core_schema.any_schema(), serialization={'exclude': {1, 3, 5}}))
    assert v.to_python(['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h']) == ['a', 'c', 'e', 'g', 'h']
