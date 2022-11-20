from pydantic_core import SchemaSerializer, core_schema


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


def test_include():
    v = SchemaSerializer(core_schema.dict_schema(serialization={'include': {'a', 'c'}}))

    assert v.to_python({'a': 1, 'b': 2, 'c': 3, 'd': 4}) == {'a': 1, 'c': 3}
    assert v.to_json({'a': 1, 'b': 2, 'c': 3, 'd': 4}) == b'{"a":1,"c":3}'

    assert v.to_python({'a': 1, 'b': 2, 'd': 4}, include={'d'}) == {'a': 1, 'd': 4}
    assert v.to_python({'a': 1, 'b': 2, 'd': 4}, include={'d': None}) == {'a': 1, 'd': 4}
    assert v.to_python({'a': 1, 'b': 2, 'd': 4}, include={'d': {1}}) == {'a': 1, 'd': 4}

    assert v.to_python({'a': 1, 'b': 2, 'd': 4, 5: 6}, include={5}) == {'a': 1, 5: 6}
    assert v.to_python({'a': 1, 'b': 2, 'd': 4, 5: 6}, mode='json', include={5}) == {'a': 1, '5': 6}
    assert v.to_json({'a': 1, 'b': 2, 'd': 4, 5: 6}, include={5}) == b'{"a":1,"5":6}'


def test_exclude():
    v = SchemaSerializer(core_schema.dict_schema(serialization={'exclude': {'a', 'c'}}))

    assert v.to_python({'a': 1, 'b': 2, 'c': 3, 'd': 4}) == {'b': 2, 'd': 4}
    assert v.to_json({'a': 1, 'b': 2, 'c': 3, 'd': 4}) == b'{"b":2,"d":4}'

    assert v.to_python({'a': 1, 'b': 2, 'c': 3, 'd': 4}, exclude={'d'}) == {'b': 2}
    assert v.to_python({'a': 1, 'b': 2, 'c': 3, 'd': 4}, exclude={'d': None}) == {'b': 2}
    assert v.to_python({'a': 1, 'b': 2, 'c': 3, 'd': 4}, exclude={'d': {1}}) == {'b': 2, 'd': 4}

    assert v.to_json({'a': 1, 'b': 2, 'c': 3, 'd': 4}, exclude={'d'}) == b'{"b":2}'
