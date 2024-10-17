import pytest

from pydantic_core import GatherInvalidDefinitionError, core_schema, gather_schemas_for_cleaning


def test_no_refs():
    p1 = core_schema.arguments_parameter('a', core_schema.int_schema())
    p2 = core_schema.arguments_parameter('b', core_schema.int_schema())
    schema = core_schema.arguments_schema([p1, p2])
    res = gather_schemas_for_cleaning(schema, definitions={}, find_meta_with_keys=None)
    assert res['inlinable_def_refs'] == {}
    assert res['recursive_refs'] == set()
    assert res['schemas_with_meta_keys'] is None


def test_simple_ref_schema():
    schema = core_schema.definition_reference_schema('ref1')
    definitions = {'ref1': core_schema.int_schema(ref='ref1')}

    res = gather_schemas_for_cleaning(schema, definitions, find_meta_with_keys=None)
    assert res['inlinable_def_refs'] == {'ref1': schema} and res['inlinable_def_refs']['ref1'] is schema
    assert res['recursive_refs'] == set()
    assert res['schemas_with_meta_keys'] is None


def test_deep_ref_schema_used_multiple_times():
    class Model:
        pass

    ref11 = core_schema.definition_reference_schema('ref1')
    ref12 = core_schema.definition_reference_schema('ref1')
    ref2 = core_schema.definition_reference_schema('ref2')

    union = core_schema.union_schema([core_schema.int_schema(), (ref11, 'ref_label')])
    tup = core_schema.tuple_schema([ref12, core_schema.str_schema()])
    schema = core_schema.model_schema(
        Model,
        core_schema.model_fields_schema(
            {'a': core_schema.model_field(union), 'b': core_schema.model_field(ref2), 'c': core_schema.model_field(tup)}
        ),
    )
    definitions = {'ref1': core_schema.str_schema(ref='ref1'), 'ref2': core_schema.bytes_schema(ref='ref2')}

    res = gather_schemas_for_cleaning(schema, definitions, find_meta_with_keys=None)
    assert res['inlinable_def_refs'] == {'ref1': None, 'ref2': ref2} and res['inlinable_def_refs']['ref2'] is ref2
    assert res['recursive_refs'] == set()
    assert res['schemas_with_meta_keys'] is None


def test_ref_in_serialization_schema():
    ref = core_schema.definition_reference_schema('ref1')
    schema = core_schema.str_schema(
        serialization=core_schema.plain_serializer_function_ser_schema(lambda v: v, return_schema=ref),
    )
    res = gather_schemas_for_cleaning(schema, definitions={'ref1': core_schema.str_schema()}, find_meta_with_keys=None)
    assert res['inlinable_def_refs'] == {'ref1': ref} and res['inlinable_def_refs']['ref1'] is ref
    assert res['recursive_refs'] == set()
    assert res['schemas_with_meta_keys'] is None


def test_recursive_ref_schema():
    ref1 = core_schema.definition_reference_schema('ref1')
    res = gather_schemas_for_cleaning(ref1, definitions={'ref1': ref1}, find_meta_with_keys=None)
    assert res['inlinable_def_refs'] == {'ref1': None}
    assert res['recursive_refs'] == {'ref1'}
    assert res['schemas_with_meta_keys'] is None


def test_deep_recursive_ref_schema():
    ref1 = core_schema.definition_reference_schema('ref1')
    ref2 = core_schema.definition_reference_schema('ref2')
    ref3 = core_schema.definition_reference_schema('ref3')

    res = gather_schemas_for_cleaning(
        core_schema.union_schema([ref1, core_schema.int_schema()]),
        definitions={
            'ref1': core_schema.union_schema([core_schema.int_schema(), ref2]),
            'ref2': core_schema.union_schema([ref3, core_schema.float_schema()]),
            'ref3': core_schema.union_schema([ref1, core_schema.str_schema()]),
        },
        find_meta_with_keys=None,
    )
    assert res['inlinable_def_refs'] == {'ref1': None, 'ref2': None, 'ref3': None}
    assert res['recursive_refs'] == {'ref1', 'ref2', 'ref3'}
    assert res['schemas_with_meta_keys'] is None


def test_find_meta():
    class Model:
        pass

    ref1 = core_schema.definition_reference_schema('ref1')

    field1 = core_schema.model_field(core_schema.str_schema())
    field1['metadata'] = {'find_meta1': 'foobar1', 'unknown': 'foobar2'}

    field2 = core_schema.model_field(core_schema.int_schema())
    field2['metadata'] = {'find_meta1': 'foobar3', 'find_meta2': 'foobar4'}

    schema = core_schema.model_schema(
        Model, core_schema.model_fields_schema({'a': field1, 'b': ref1, 'c': core_schema.float_schema()})
    )
    res = gather_schemas_for_cleaning(
        schema, definitions={'ref1': field2}, find_meta_with_keys={'find_meta1', 'find_meta2'}
    )
    assert res['inlinable_def_refs'] == {'ref1': ref1} and res['inlinable_def_refs']['ref1'] is ref1
    assert res['recursive_refs'] == set()
    assert res['schemas_with_meta_keys'] == {'find_meta1': [field1, field2], 'find_meta2': [field2]}
    assert res['schemas_with_meta_keys']['find_meta1'][0] is field1
    assert res['schemas_with_meta_keys']['find_meta1'][1] is field2
    assert res['schemas_with_meta_keys']['find_meta2'][0] is field2


def test_unknown_ref():
    ref1 = core_schema.definition_reference_schema('ref1')
    schema = core_schema.tuple_schema([core_schema.int_schema(), ref1])
    with pytest.raises(GatherInvalidDefinitionError, match='ref1'):
        gather_schemas_for_cleaning(schema, definitions={}, find_meta_with_keys=None)


def test_no_duplicate_ref_instances_gathered():
    schema1 = core_schema.tuple_schema([core_schema.str_schema(), core_schema.int_schema()])
    schema2 = core_schema.tuple_schema(
        [core_schema.definition_reference_schema('ref1'), core_schema.definition_reference_schema('ref1')]
    )
    schema3 = core_schema.tuple_schema(
        [core_schema.definition_reference_schema('ref2'), core_schema.definition_reference_schema('ref2')]
    )
    definitions = {'ref1': schema1, 'ref2': schema2}

    res = gather_schemas_for_cleaning(schema3, definitions=definitions, find_meta_with_keys=None)
    assert res['inlinable_def_refs'] == {'ref1': None, 'ref2': None}
    assert res['recursive_refs'] == set()
    assert res['schemas_with_meta_keys'] is None
