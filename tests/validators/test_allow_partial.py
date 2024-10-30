import pytest
from dirty_equals import IsStrictDict
from inline_snapshot import snapshot

from generate_self_schema import core_schema
from pydantic_core import SchemaValidator, ValidationError


def test_list():
    v = SchemaValidator(
        core_schema.list_schema(
            core_schema.tuple_positional_schema([core_schema.int_schema(), core_schema.int_schema()]),
        )
    )
    assert v.validate_python([[1, 2], [3, 4]]) == [(1, 2), (3, 4)]
    assert v.validate_python([[1, 2], [3, 4]], allow_partial=True) == [(1, 2), (3, 4)]
    with pytest.raises(ValidationError) as exc_info:
        v.validate_python([[1, 2], 'wrong'])
    assert exc_info.value.errors(include_url=False) == snapshot(
        [
            {
                'type': 'tuple_type',
                'loc': (1,),
                'msg': 'Input should be a valid tuple',
                'input': 'wrong',
            }
        ]
    )
    assert v.validate_python([[1, 2], 'wrong'], allow_partial=True) == [(1, 2)]
    assert v.validate_python([[1, 2], []], allow_partial=True) == [(1, 2)]
    assert v.validate_python([[1, 2], [3]], allow_partial=True) == [(1, 2)]
    assert v.validate_python([[1, 2], [3, 'x']], allow_partial=True) == [(1, 2)]
    with pytest.raises(ValidationError, match='Input should be a valid tuple'):
        v.validate_python([[1, 2], 'wrong', [3, 4]])
    with pytest.raises(ValidationError, match='Input should be a valid tuple'):
        v.validate_python([[1, 2], 'wrong', 'wrong'])
    assert v.validate_json(b'[[1, 2], [3, 4]]', allow_partial=True) == [(1, 2), (3, 4)]
    assert v.validate_json(b'[[1, 2], [3,', allow_partial=True) == [(1, 2)]


def test_list_partial_nested():
    v = SchemaValidator(
        core_schema.tuple_positional_schema(
            [core_schema.int_schema(), core_schema.list_schema(core_schema.int_schema())]
        ),
    )
    assert v.validate_python([1, [2, 3]]) == (1, [2, 3])
    assert v.validate_python([1, [2, 3]], allow_partial=True) == (1, [2, 3])
    with pytest.raises(ValidationError) as exc_info:
        v.validate_python((1, [2, 3, 'x']))
    assert exc_info.value.errors(include_url=False) == snapshot(
        [
            {
                'type': 'int_parsing',
                'loc': (1, 2),
                'msg': 'Input should be a valid integer, unable to parse string as an integer',
                'input': 'x',
            }
        ]
    )
    assert v.validate_python((1, [2, 3, 'x']), allow_partial=True) == (1, [2, 3])


@pytest.mark.parametrize('collection_type', [core_schema.set_schema, core_schema.frozenset_schema])
def test_set_frozenset(collection_type):
    v = SchemaValidator(
        collection_type(
            core_schema.tuple_positional_schema([core_schema.int_schema(), core_schema.int_schema()]),
        )
    )
    assert v.validate_python([[1, 2], [3, 4]]) == snapshot({(1, 2), (3, 4)})
    assert v.validate_python([[1, 2], [3, 4]], allow_partial=True) == snapshot({(1, 2), (3, 4)})
    with pytest.raises(ValidationError) as exc_info:
        v.validate_python([[1, 2], 'wrong'])
    assert exc_info.value.errors(include_url=False) == snapshot(
        [
            {
                'type': 'tuple_type',
                'loc': (1,),
                'msg': 'Input should be a valid tuple',
                'input': 'wrong',
            }
        ]
    )
    assert v.validate_python([[1, 2], 'wrong'], allow_partial=True) == snapshot({(1, 2)})
    assert v.validate_python([[1, 2], [3, 4], 'wrong'], allow_partial=True) == snapshot({(1, 2), (3, 4)})
    assert v.validate_python([[1, 2], []], allow_partial=True) == snapshot({(1, 2)})
    assert v.validate_python([[1, 2], [3]], allow_partial=True) == snapshot({(1, 2)})
    assert v.validate_python([[1, 2], [3, 'x']], allow_partial=True) == snapshot({(1, 2)})
    with pytest.raises(ValidationError, match='Input should be a valid tuple'):
        v.validate_python([[1, 2], 'wrong', [3, 4]])
    with pytest.raises(ValidationError, match='Input should be a valid tuple'):
        v.validate_python([[1, 2], 'wrong', 'wrong'])


def test_dict():
    v = SchemaValidator(core_schema.dict_schema(core_schema.int_schema(), core_schema.int_schema()))
    assert v.validate_python({'1': 2, 3: '4'}) == snapshot({1: 2, 3: 4})
    assert v.validate_python({'1': 2, 3: '4'}, allow_partial=True) == snapshot({1: 2, 3: 4})
    with pytest.raises(ValidationError) as exc_info:
        v.validate_python({'1': 2, 3: 'wrong'})
    assert exc_info.value.errors(include_url=False) == snapshot(
        [
            {
                'type': 'int_parsing',
                'loc': (3,),
                'msg': 'Input should be a valid integer, unable to parse string as an integer',
                'input': 'wrong',
            }
        ]
    )
    assert v.validate_python({'1': 2, 3: 'wrong'}, allow_partial=True) == snapshot({1: 2})
    assert v.validate_python({'1': 2, 3: 4, 5: '6', 7: 'x'}, allow_partial=True) == snapshot({1: 2, 3: 4, 5: 6})
    with pytest.raises(ValidationError, match='Input should be a valid integer'):
        v.validate_python({'1': 2, 3: 4, 5: 'x', 7: '8'})
    with pytest.raises(ValidationError, match='Input should be a valid integer'):
        v.validate_python({'1': 2, 3: 4, 5: 'x', 7: 'x'})
    with pytest.raises(ValidationError, match='Input should be a valid integer'):
        v.validate_python({'1': 2, 3: 4, 'x': 6})


def test_partial_typed_dict():
    v = SchemaValidator(
        core_schema.typed_dict_schema(
            {
                'a': core_schema.typed_dict_field(core_schema.int_schema(gt=10)),
                'b': core_schema.typed_dict_field(core_schema.int_schema(gt=10)),
                'c': core_schema.typed_dict_field(core_schema.int_schema(gt=10)),
            },
            total=False,
        )
    )

    assert v.validate_python({'a': 11, 'b': '12', 'c': 13}) == snapshot(IsStrictDict(a=11, b=12, c=13))
    assert v.validate_python({'a': 11, 'c': 13, 'b': '12'}) == snapshot(IsStrictDict(a=11, b=12, c=13))

    assert v.validate_python({'a': 11, 'b': '12', 'c': 13}, allow_partial=True) == snapshot({'a': 11, 'b': 12, 'c': 13})
    with pytest.raises(ValidationError) as exc_info:
        v.validate_python({'a': 11, 'b': '12', 'c': 1})
    assert exc_info.value.errors(include_url=False) == snapshot(
        [
            {
                'type': 'greater_than',
                'loc': ('c',),
                'msg': 'Input should be greater than 10',
                'input': 1,
                'ctx': {'gt': 10},
            }
        ]
    )
    assert v.validate_python({'a': 11, 'b': '12', 'c': 1}, allow_partial=True) == snapshot(IsStrictDict(a=11, b=12))
    assert v.validate_python({'a': 11, 'c': 13, 'b': 1}, allow_partial=True) == snapshot(IsStrictDict(a=11, c=13))
    with pytest.raises(ValidationError) as exc_info:
        v.validate_python({'a': 11, 'c': 1, 'b': 12}, allow_partial=True)
    assert exc_info.value.errors(include_url=False) == snapshot(
        [
            {
                'type': 'greater_than',
                'loc': ('c',),
                'msg': 'Input should be greater than 10',
                'input': 1,
                'ctx': {'gt': 10},
            }
        ]
    )

    # validate strings
    assert v.validate_strings({'a': '11', 'b': '22'}) == snapshot({'a': 11, 'b': 22})
    with pytest.raises(ValidationError, match='Input should be greater than 10'):
        v.validate_strings({'a': '11', 'b': '2'})
    assert v.validate_strings({'a': '11', 'b': '2'}, allow_partial=True) == snapshot({'a': 11})

    assert v.validate_json(b'{"b": "12", "a": 11, "c": 13}', allow_partial=True) == IsStrictDict(a=11, b=12, c=13)
    assert v.validate_json(b'{"b": "12", "a": 11, "c": 13', allow_partial=True) == IsStrictDict(a=11, b=12, c=13)
    assert v.validate_json(b'{"a": 11, "b": "12", "c": 1', allow_partial=True) == IsStrictDict(a=11, b=12)
    assert v.validate_json(b'{"a": 11, "b": "12", "c":', allow_partial=True) == IsStrictDict(a=11, b=12)
    assert v.validate_json(b'{"a": 11, "b": "12", "c"', allow_partial=True) == IsStrictDict(a=11, b=12)
    assert v.validate_json(b'{"a": 11, "b": "12", "c', allow_partial=True) == IsStrictDict(a=11, b=12)
    assert v.validate_json(b'{"a": 11, "b": "12", "', allow_partial=True) == IsStrictDict(a=11, b=12)
    assert v.validate_json(b'{"a": 11, "b": "12", ', allow_partial=True) == IsStrictDict(a=11, b=12)
    assert v.validate_json(b'{"a": 11, "b": "12",', allow_partial=True) == IsStrictDict(a=11, b=12)
    assert v.validate_json(b'{"a": 11, "b": "12"', allow_partial=True) == IsStrictDict(a=11, b=12)


def test_non_partial_typed_dict():
    v = SchemaValidator(
        core_schema.typed_dict_schema(
            {
                'a': core_schema.typed_dict_field(core_schema.int_schema(gt=10)),
                'b': core_schema.typed_dict_field(core_schema.int_schema(gt=10), required=True),
                'c': core_schema.typed_dict_field(core_schema.int_schema(gt=10)),
            },
            total=False,
        )
    )

    assert v.validate_python({'a': 11, 'b': '12', 'c': 13}) == snapshot({'a': 11, 'b': 12, 'c': 13})
    with pytest.raises(ValidationError, match='Input should be greater than 10'):
        v.validate_python({'a': 11, 'b': '12', 'c': 1})
    with pytest.raises(ValidationError, match='Input should be greater than 10'):
        v.validate_python({'a': 11, 'b': '12', 'c': 1}, allow_partial=False)


def test_double_nested():
    v = SchemaValidator(
        core_schema.typed_dict_schema(
            {
                'a': core_schema.typed_dict_field(core_schema.int_schema(gt=10)),
                'b': core_schema.typed_dict_field(
                    core_schema.list_schema(
                        core_schema.dict_schema(core_schema.str_schema(), core_schema.int_schema(ge=10))
                    )
                ),
            },
            total=False,
        )
    )
    assert v.validate_python({'a': 11, 'b': [{'a': 10, 'b': 20}, {'a': 30, 'b': 40}]}) == snapshot(
        {'a': 11, 'b': [{'a': 10, 'b': 20}, {'a': 30, 'b': 40}]}
    )
    assert v.validate_python({'a': 11, 'b': [{'a': 10, 'b': 20}, {'a': 30, 'b': 4}]}, allow_partial=True) == snapshot(
        {'a': 11, 'b': [{'a': 10, 'b': 20}, {'a': 30}]}
    )
    assert v.validate_python({'a': 11, 'b': [{'a': 10, 'b': 20}, {'a': 30, 123: 4}]}, allow_partial=True) == snapshot(
        {'a': 11, 'b': [{'a': 10, 'b': 20}]}
    )
    # this is not the intended behaviour, but it's okay
    assert v.validate_python({'a': 11, 'b': [{'a': 10, 'b': 2}, {'a': 30}]}, allow_partial=True) == snapshot(
        {'a': 11, 'b': [{'a': 10}, {'a': 30}]}
    )
    assert v.validate_python({'a': 11, 'b': [{'a': 1, 'b': 20}, {'a': 3, 'b': 40}]}, allow_partial=True) == snapshot(
        {'a': 11}
    )
    json = b'{"a": 11, "b": [{"a": 10, "b": 20}, {"a": 30, "b": 40}]}'
    assert v.validate_json(json, allow_partial=True) == snapshot(
        {'a': 11, 'b': [{'a': 10, 'b': 20}, {'a': 30, 'b': 40}]}
    )
    for i in range(1, len(json)):
        value = v.validate_json(json[:i], allow_partial=True)
        assert isinstance(value, dict)
