import pytest

from pydantic_core import SchemaError, SchemaSerializer, core_schema

from ..conftest import plain_repr


def test_custom_ser():
    s = SchemaSerializer(
        core_schema.list_schema(core_schema.definition_reference_schema('foobar')),
        None,
        [core_schema.int_schema(ref='foobar', serialization=core_schema.to_string_ser_schema(when_used='always'))],
    )
    assert s.to_python([1, 2, 3]) == ['1', '2', '3']


def test_ignored_def():
    s = SchemaSerializer(
        core_schema.list_schema(core_schema.int_schema()),
        None,
        [core_schema.int_schema(ref='foobar', serialization=core_schema.to_string_ser_schema(when_used='always'))],
    )
    assert s.to_python([1, 2, 3]) == [1, 2, 3]
    assert plain_repr(s).endswith('slots=[])')


def test_def_error():
    with pytest.raises(SchemaError) as exc_info:
        SchemaSerializer(
            core_schema.list_schema(core_schema.definition_reference_schema('foobar')),
            None,
            [core_schema.int_schema(ref='foobar'), {'type': 'wrong'}],
        )

    assert exc_info.value.args[0].startswith(
        "Invalid Schema:\ndefinitions -> 1\n  Input tag 'wrong' found using self-schema"
    )
