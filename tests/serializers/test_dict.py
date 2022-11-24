import pytest

from pydantic_core import SchemaError, SchemaSerializer, core_schema


def test_dict_str_int():
    v = SchemaSerializer(core_schema.dict_schema(core_schema.string_schema(), core_schema.int_schema()))
    assert v.to_python({'a': 1, 'b': 2, 'c': 3}) == {'a': 1, 'b': 2, 'c': 3}
    assert v.to_python({'a': 1, 'b': 2, 'c': 3}, mode='json') == {'a': 1, 'b': 2, 'c': 3}
    assert v.to_json({'a': 1, 'b': 2, 'c': 3}) == b'{"a":1,"b":2,"c":3}'

    assert v.to_json({'a': 1, 'b': 2, 'c': 3}, indent=2) == b'{\n  "a": 1,\n  "b": 2,\n  "c": 3\n}'


def test_dict_any_any():
    v = SchemaSerializer(core_schema.dict_schema())
    assert v.to_python({'a': 1, b'b': 2, 33: 3}) == {'a': 1, b'b': 2, 33: 3}
    assert v.to_python({'a': 1, b'b': 2, 33: 3, True: 4}, mode='json') == {'a': 1, 'b': 2, '33': 3, 'true': 4}
    assert v.to_json({'a': 1, b'b': 2, 33: 3, True: 4}) == b'{"a":1,"b":2,"33":3,"true":4}'

    assert v.to_python({(1, 2): 3}) == {(1, 2): 3}
    assert v.to_python({(1, 2): 3}, mode='json') == {'(1, 2)': 3}
    assert v.to_json({(1, 2): 3}) == b'{"(1, 2)":3}'


def test_include():
    v = SchemaSerializer(core_schema.dict_schema(serialization=core_schema.inc_ex_dict_schema(include={'a', 'c'})))

    assert v.to_python({'a': 1, 'b': 2, 'c': 3, 'd': 4}) == {'a': 1, 'c': 3}
    assert v.to_json({'a': 1, 'b': 2, 'c': 3, 'd': 4}) == b'{"a":1,"c":3}'

    assert v.to_python({'a': 1, 'b': 2, 'd': 4}, include={'d'}) == {'a': 1, 'd': 4}
    assert v.to_python({'a': 1, 'b': 2, 'd': 4}, include={'d': None}) == {'a': 1, 'd': 4}
    assert v.to_python({'a': 1, 'b': 2, 'd': 4}, include={'d': {1}}) == {'a': 1, 'd': 4}

    assert v.to_python({'a': 1, 'b': 2, 'd': 4, 5: 6}, include={5}) == {'a': 1, 5: 6}
    assert v.to_python({'a': 1, 'b': 2, 'd': 4, 5: 6}, mode='json', include={5}) == {'a': 1, '5': 6}
    assert v.to_json({'a': 1, 'b': 2, 'd': 4, 5: 6}, include={5}) == b'{"a":1,"5":6}'


def test_exclude():
    v = SchemaSerializer(core_schema.dict_schema(serialization=core_schema.inc_ex_dict_schema(exclude={'a', 'c'})))

    assert v.to_python({'a': 1, 'b': 2, 'c': 3, 'd': 4}) == {'b': 2, 'd': 4}
    assert v.to_json({'a': 1, 'b': 2, 'c': 3, 'd': 4}) == b'{"b":2,"d":4}'

    assert v.to_python({'a': 1, 'b': 2, 'c': 3, 'd': 4}, exclude={'d'}) == {'b': 2}
    assert v.to_python({'a': 1, 'b': 2, 'c': 3, 'd': 4}, exclude={'d': None}) == {'b': 2}
    assert v.to_python({'a': 1, 'b': 2, 'c': 3, 'd': 4}, exclude={'d': {1}}) == {'b': 2, 'd': 4}

    assert v.to_json({'a': 1, 'b': 2, 'c': 3, 'd': 4}, exclude={'d'}) == b'{"b":2}'


def test_include_exclude():
    v = SchemaSerializer(
        core_schema.dict_schema(
            core_schema.any_schema(),
            serialization=core_schema.inc_ex_dict_schema(include={'1', '3', '5'}, exclude={'5', '6'}),
        )
    )

    assert v.to_python({'0': 0, '1': 1, '2': 2, '3': 3, '4': 4, '5': 5, '6': 6, '7': 7}) == {'1': 1, '3': 3}


def test_include_exclude_int():
    v = SchemaSerializer(
        core_schema.dict_schema(
            core_schema.any_schema(), serialization=core_schema.inc_ex_dict_schema(include={1, 3, 5}, exclude={5, 6})
        )
    )

    assert v.to_python({0: 0, 1: 1, 2: 2, 3: 3, 4: 4, 5: 5, 6: 6, 7: 7}) == {1: 1, 3: 3}


def test_include_exclude_runtime():
    v = SchemaSerializer(
        core_schema.dict_schema(
            core_schema.any_schema(), serialization=core_schema.inc_ex_dict_schema(exclude={'0', '1'})
        )
    )
    assert v.to_python({'0': 0, '1': 1, '2': 2, '3': 3}, include={'1', '2'}) == {'2': 2}


def test_include_exclude_runtime_int():
    v = SchemaSerializer(
        core_schema.dict_schema(core_schema.any_schema(), serialization=core_schema.inc_ex_dict_schema(exclude={0, 1}))
    )
    assert v.to_python({0: 0, 1: 1, 2: 2, 3: 3}, include={1, 2}) == {2: 2}


@pytest.mark.parametrize(
    'include_value,error_msg',
    [
        ('foobar', 'Input should be a valid set'),
        ({'a': 'dict'}, 'Input should be a valid set'),
        ({4.2}, 'Input should be a valid integer, got a number with a fractional part'),
    ],
)
def test_include_error(include_value, error_msg):
    with pytest.raises(SchemaError, match=error_msg):
        SchemaSerializer(core_schema.dict_schema(serialization=core_schema.inc_ex_dict_schema(include=include_value)))
