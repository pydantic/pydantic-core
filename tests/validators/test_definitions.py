import pytest

from pydantic_core import SchemaError, SchemaValidator, core_schema

from ..conftest import plain_repr


def test_list_with_def():
    v = SchemaValidator(
        core_schema.list_schema(core_schema.definition_reference_schema('foobar')),
        None,
        [core_schema.int_schema(ref='foobar')],
    )
    assert v.validate_python([1, 2, '3']) == [1, 2, 3]
    assert v.validate_json(b'[1, 2, "3"]') == [1, 2, 3]
    r = plain_repr(v)
    assert r.startswith('SchemaValidator(name="list[int]",')
    assert r.endswith('slots=[Int(IntValidator{strict:false}),])')


def test_ignored_def():
    v = SchemaValidator(core_schema.list_schema(core_schema.int_schema()), None, [core_schema.int_schema(ref='foobar')])
    assert v.validate_python([1, 2, '3']) == [1, 2, 3]
    r = plain_repr(v)
    assert r.startswith('SchemaValidator(name="list[int]",')
    assert r.endswith('slots=[])')


def test_def_error():
    with pytest.raises(SchemaError) as exc_info:
        SchemaValidator(
            core_schema.list_schema(core_schema.definition_reference_schema('foobar')),
            None,
            [core_schema.int_schema(ref='foobar'), {'type': 'wrong'}],
        )

    assert exc_info.value.args[0].startswith(
        "Invalid Schema:\ndefinitions -> 1\n  Input tag 'wrong' found using self-schema"
    )
