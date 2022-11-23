import pytest

from pydantic_core import SchemaError, SchemaSerializer, core_schema


def test_list_any():
    v = SchemaSerializer(core_schema.list_schema(core_schema.any_schema()))
    assert v.to_python(['a', 'b', 'c']) == ['a', 'b', 'c']
    assert v.to_python(['a', 'b', 'c'], mode='json') == ['a', 'b', 'c']
    assert v.to_json(['a', 'b', 'c']) == b'["a","b","c"]'

    assert v.to_json(['a', 'b', 'c'], indent=2) == b'[\n  "a",\n  "b",\n  "c"\n]'


def test_list_fallback():
    v = SchemaSerializer(core_schema.list_schema(core_schema.any_schema()))
    with pytest.warns(UserWarning, match='Expected `list` but got `str` - filtering via include/exclude unavailable'):
        assert v.to_python('apple') == 'apple'

    with pytest.warns(UserWarning) as warning_info:
        assert v.to_json('apple') == b'"apple"'
    assert [w.message.args[0] for w in warning_info.list] == [
        'Pydantic serializer warnings:\n  Expected `list` but got `str` - filtering via include/exclude unavailable'
    ]

    with pytest.warns(UserWarning, match='Expected `list` but got `bytes` - filtering via include/exclude unavailable'):
        assert v.to_json(b'apple') == b'"apple"'

    with pytest.warns(UserWarning, match='Expected `list` but got `tuple` - filtering via include/exclude unavailable'):
        assert v.to_python((1, 2, 3)) == (1, 2, 3)

    # # even though we're in the fallback state, non JSON types should still be converted to JSON here
    with pytest.warns(UserWarning, match='Expected `list` but got `tuple` - filtering via include/exclude unavailable'):
        assert v.to_python((1, 2, 3), mode='json') == [1, 2, 3]


def test_list_str_fallback():
    v = SchemaSerializer(core_schema.list_schema(core_schema.string_schema()))
    with pytest.warns(UserWarning) as warning_info:
        assert v.to_json([1, 2, 3]) == b'[1,2,3]'
    assert [w.message.args[0] for w in warning_info.list] == [
        'Pydantic serializer warnings:\n'
        '  Expected `str` but got `int` - slight slowdown possible\n'
        '  Expected `str` but got `int` - slight slowdown possible\n'
        '  Expected `str` but got `int` - slight slowdown possible'
    ]


def test_tuple_any():
    v = SchemaSerializer(core_schema.tuple_variable_schema(core_schema.any_schema()))
    assert v.to_python(('a', 'b', 'c')) == ('a', 'b', 'c')
    assert v.to_python(('a', 'b', 'c'), mode='json') == ['a', 'b', 'c']
    assert v.to_json(('a', 'b', 'c')) == b'["a","b","c"]'

    assert v.to_json(('a', 'b', 'c'), indent=2) == b'[\n  "a",\n  "b",\n  "c"\n]'


def as_list(*items):
    return list(items)


def as_tuple(*items):
    return tuple(items)


@pytest.mark.parametrize(
    'schema_func,seq_f', [(core_schema.list_schema, as_list), (core_schema.tuple_variable_schema, as_tuple)]
)
def test_include(schema_func, seq_f):
    v = SchemaSerializer(
        schema_func(core_schema.any_schema(), serialization=core_schema.inc_ex_ser_schema(include={1, 3, 5}))
    )
    assert v.to_python(seq_f(0, 1, 2, 3)) == seq_f(1, 3)
    assert v.to_python(seq_f('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h')) == seq_f('b', 'd', 'f')
    assert v.to_python(seq_f('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'), mode='json') == ['b', 'd', 'f']
    assert v.to_json(seq_f('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h')) == b'["b","d","f"]'
    # the two include lists are now combined via UNION! unlike in pydantic v1
    assert v.to_python(seq_f('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'), include={6}) == seq_f('b', 'd', 'f', 'g')
    assert v.to_json(seq_f('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'), include={6}) == b'["b","d","f","g"]'
    assert v.to_python(seq_f('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'), include={6: None}) == seq_f('b', 'd', 'f', 'g')


@pytest.mark.parametrize(
    'schema_func,seq_f', [(core_schema.list_schema, as_list), (core_schema.tuple_variable_schema, as_tuple)]
)
def test_include_dict(schema_func, seq_f):
    v = SchemaSerializer(
        schema_func(
            core_schema.any_schema(), serialization=core_schema.inc_ex_ser_schema(include={1: None, 3: None, 5: {42}})
        )
    )
    assert v.to_python(seq_f(0, 1, 2, 3, 4)) == seq_f(1, 3)
    assert v.to_python(seq_f('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h')) == seq_f('b', 'd', 'f')
    assert v.to_python(seq_f(0, 1, 2, 3, 4), include={2: None}) == seq_f(1, 2, 3)
    assert v.to_python(seq_f(0, 1, 2, 3, 4), include={2: {1, 2}}) == seq_f(1, 2, 3)
    assert v.to_python(seq_f(0, 1, 2, 3, 4), include={2}) == seq_f(1, 2, 3)


@pytest.mark.parametrize(
    'schema_func,seq_f', [(core_schema.list_schema, as_list), (core_schema.tuple_variable_schema, as_tuple)]
)
def test_exclude(schema_func, seq_f):
    v = SchemaSerializer(
        schema_func(core_schema.any_schema(), serialization=core_schema.inc_ex_ser_schema(exclude={1, 3, 5}))
    )
    assert v.to_python(seq_f(0, 1, 2, 3)) == seq_f(0, 2)
    assert v.to_python(seq_f('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h')) == seq_f('a', 'c', 'e', 'g', 'h')
    assert v.to_python(seq_f('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'), mode='json') == ['a', 'c', 'e', 'g', 'h']
    assert v.to_json(seq_f('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h')) == b'["a","c","e","g","h"]'
    # the two exclude lists are combined via union as they used to be
    assert v.to_python(seq_f('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'), exclude={6}) == seq_f('a', 'c', 'e', 'h')
    assert v.to_json(seq_f('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'), exclude={6}) == b'["a","c","e","h"]'


def test_include_exclude():
    v = SchemaSerializer(
        core_schema.list_schema(
            core_schema.any_schema(), serialization=core_schema.inc_ex_ser_schema(include={1, 3, 5}, exclude={5, 6})
        )
    )
    assert v.to_python([0, 1, 2, 3, 4, 5, 6, 7]) == [1, 3]


def test_include_exclude_runtime():
    v = SchemaSerializer(
        core_schema.list_schema(core_schema.any_schema(), serialization=core_schema.inc_ex_ser_schema(exclude={0, 1}))
    )
    assert v.to_python([0, 1, 2, 3], include={1, 2}) == [2]


@pytest.mark.parametrize('schema_func', [core_schema.list_schema, core_schema.tuple_variable_schema])
def test_include_error(schema_func):
    with pytest.raises(SchemaError, match='Input should be a valid dictionary'):
        SchemaSerializer(
            schema_func(core_schema.any_schema(), serialization=core_schema.inc_ex_ser_schema(include='foobar'))
        )


@pytest.mark.parametrize(
    'schema_func,seq_f', [(core_schema.list_schema, as_list), (core_schema.tuple_variable_schema, as_tuple)]
)
def test_include_error_call_time(schema_func, seq_f):
    v = SchemaSerializer(schema_func(core_schema.any_schema()))
    with pytest.raises(TypeError, match='`include` and `exclude` inputs must be sets or dicts.'):
        v.to_python(seq_f(0, 1, 2, 3), include=[1, 3, 5])


def test_tuple_fallback():
    v = SchemaSerializer(core_schema.tuple_variable_schema(core_schema.any_schema()))
    with pytest.warns(UserWarning, match='Expected `tuple` but got `str` - filtering via include/exclude unavailable'):
        assert v.to_python('apple') == 'apple'

    with pytest.warns(UserWarning) as warning_info:
        assert v.to_json([1, 2, 3]) == b'[1,2,3]'
    assert [w.message.args[0] for w in warning_info.list] == [
        'Pydantic serializer warnings:\n  Expected `tuple` but got `list` - filtering via include/exclude unavailable'
    ]

    with pytest.warns(
        UserWarning, match='Expected `tuple` but got `bytes` - filtering via include/exclude unavailable'
    ):
        assert v.to_json(b'apple') == b'"apple"'

    assert v.to_python((1, 2, 3)) == (1, 2, 3)

    # # even though we're in the fallback state, non JSON types should still be converted to JSON here
    with pytest.warns(UserWarning, match='Expected `tuple` but got `list` - filtering via include/exclude unavailable'):
        assert v.to_python([1, 2, 3], mode='json') == [1, 2, 3]
