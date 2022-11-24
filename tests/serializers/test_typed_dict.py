import json

import pytest
from dirty_equals import IsStrictDict

from pydantic_core import SchemaSerializer, core_schema


def test_typed_dict():
    v = SchemaSerializer(
        core_schema.typed_dict_schema(
            {
                'foo': core_schema.typed_dict_field(core_schema.int_schema()),
                'bar': core_schema.typed_dict_field(core_schema.bytes_schema()),
            },
            extra_behavior='ignore',  # this is the default
        )
    )
    assert v.to_python({'foo': 1, 'bar': b'more'}) == IsStrictDict(foo=1, bar=b'more')
    assert v.to_python({'bar': b'more', 'foo': 1}) == IsStrictDict(foo=1, bar=b'more')
    assert v.to_python({'foo': 1, 'bar': b'more', 'c': 3}) == IsStrictDict(foo=1, bar=b'more')
    assert v.to_python({'bar': b'more', 'foo': 1, 'c': 3}, mode='json') == IsStrictDict(foo=1, bar='more')

    assert v.to_json({'bar': b'more', 'foo': 1, 'c': 3}) == b'{"foo":1,"bar":"more"}'


def test_typed_dict_allow_extra():
    v = SchemaSerializer(
        core_schema.typed_dict_schema(
            {
                'foo': core_schema.typed_dict_field(core_schema.int_schema()),
                'bar': core_schema.typed_dict_field(core_schema.bytes_schema()),
            },
            extra_behavior='allow',
        )
    )
    # extra fields go last but retain their order
    assert v.to_python({'bar': b'more', 'b': 3, 'foo': 1, 'a': 4}) == IsStrictDict(foo=1, bar=b'more', b=3, a=4)
    assert v.to_python({'bar': b'more', 'c': 3, 'foo': 1}, mode='json') == IsStrictDict(foo=1, bar='more', c=3)

    assert v.to_json({'bar': b'more', 'c': 3, 'foo': 1, 'cc': 4}) == b'{"foo":1,"bar":"more","c":3,"cc":4}'


@pytest.mark.parametrize(
    'params',
    [
        dict(include=None, exclude=None, expected={'0': 0, '1': 1, '2': 2, '3': 3}),
        dict(include={'0', '1'}, exclude=None, expected={'0': 0, '1': 1}),
        dict(include={'0': None, '1': None}, exclude=None, expected={'0': 0, '1': 1}),
        dict(include={'0': {1}, '1': {1}}, exclude=None, expected={'0': 0, '1': 1}),
        dict(include=None, exclude={'0', '1'}, expected={'2': 2, '3': 3}),
        dict(include=None, exclude={'0': None, '1': None}, expected={'2': 2, '3': 3}),
        dict(include={'0', '1'}, exclude={'1', '2'}, expected={'0': 0}),
        dict(include=None, exclude={'3': {1}}, expected={'0': 0, '1': 1, '2': 2, '3': 3}),
        dict(include={'0', '1'}, exclude={'3': {1}}, expected={'0': 0, '1': 1}),
        dict(include={'0', '1'}, exclude={'1': {1}}, expected={'0': 0, '1': 1}),
        dict(include={'0', '1'}, exclude={'1': None}, expected={'0': 0}),
    ],
)
def test_include_exclude_args(params):
    s = SchemaSerializer(
        core_schema.typed_dict_schema(
            {
                '0': core_schema.typed_dict_field(core_schema.int_schema()),
                '1': core_schema.typed_dict_field(core_schema.int_schema()),
                '2': core_schema.typed_dict_field(core_schema.int_schema()),
                '3': core_schema.typed_dict_field(core_schema.int_schema()),
            }
        )
    )

    # user IsStrictDict to check dict order
    include, exclude, expected = params['include'], params['exclude'], IsStrictDict(params['expected'])
    value = {'0': 0, '1': 1, '2': 2, '3': 3}
    assert s.to_python(value, include=include, exclude=exclude) == expected
    assert s.to_python(value, mode='json', include=include, exclude=exclude) == expected
    assert json.loads(s.to_json(value, include=include, exclude=exclude)) == expected


def test_include_exclude_schema():
    s = SchemaSerializer(
        core_schema.typed_dict_schema(
            {
                '0': core_schema.typed_dict_field(core_schema.int_schema(), serialization_exclude=True),
                '1': core_schema.typed_dict_field(core_schema.int_schema()),
                '2': core_schema.typed_dict_field(core_schema.int_schema(), serialization_exclude=True),
                '3': core_schema.typed_dict_field(core_schema.int_schema(), serialization_exclude=False),
            }
        )
    )
    value = {'0': 0, '1': 1, '2': 2, '3': 3}
    assert s.to_python(value) == {'1': 1, '3': 3}
    assert s.to_python(value, mode='json') == {'1': 1, '3': 3}
    assert json.loads(s.to_json(value)) == {'1': 1, '3': 3}
