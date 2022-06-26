from datetime import date, datetime, time

import pytest

from pydantic_core import SchemaValidator

from .complete_schema import input_data_lax, input_data_strict, pydantic_model, schema
from .test_micro_benchmarks import skip_pydantic


def test_complete_core_test():
    lax_schema = schema()
    class_type = lax_schema['class_type']
    lax_validator = SchemaValidator(lax_schema)
    output = lax_validator.validate_python(input_data_lax())
    assert isinstance(output, class_type)
    assert len(output.__fields_set__) == 39
    output_dict = output.__dict__
    assert output_dict == {
        'field_str': 'fo',
        'field_str_con': 'fooba',
        'field_int': 1,
        'field_int_con': 8,
        'field_float': 1.0,
        'field_float_con': 10.0,
        'field_bool': True,
        'field_bytes': b'foobar',
        'field_bytes_con': b'foobar',
        'field_date': date(2010, 2, 3),
        'field_date_con': date(2020, 1, 1),
        'field_time': time(12, 0),
        'field_time_con': time(12, 0),
        'field_datetime': datetime(2020, 1, 1, 12, 13, 14),
        'field_datetime_con': datetime(2020, 1, 1),
        'field_list_any': ['a', b'b', True, 1.0, None],
        'field_list_str': ['a', 'b', 'c'],
        'field_list_str_con': ['a', 'b', 'c'],
        'field_set_any': {b'b', True, 'a', None},
        'field_set_int': {1, 2, 3},
        'field_set_int_con': {1, 2, 3},
        'field_frozenset_any': frozenset({b'b', True, 'a', None}),
        'field_frozenset_bytes': frozenset({b'b', b'a', b'c'}),
        'field_frozenset_bytes_con': frozenset({b'b', b'a', b'c'}),
        'field_tuple_var_len_any': ('a', b'b', True, 1.0, None),
        'field_tuple_var_len_float': (1.0, 2.0, 3.0),
        'field_tuple_var_len_float_con': (1.0, 2.0, 3.0),
        'field_tuple_fix_len': ('a', 1, 1.0, True),
        'field_dict_any': {'a': 'b', 1: 1.0},
        'field_dict_str_float': {'a': 1.0, 'b': 2.0, 'c': 3.0},
        'field_literal_1_int': 1,
        'field_literal_1_str': 'foobar',
        'field_literal_mult_int': 3,
        'field_literal_mult_str': 'foo',
        'field_literal_assorted': 'foo',
        'field_list_nullable_int': [1, None, 2, None, 3, None, 4, None],
        'field_union': {'field_str': 'foo', 'field_int': 1, 'field_float': 1.0},
        'field_functions_model': {
            'field_before': 'foo Changed',
            'field_after': 'foo Changed',
            'field_wrap': 'Input foo Changed',
            'field_plain': 'foo Changed',
        },
        'field_recursive': {
            'name': 'foo',
            'sub_branch': {'name': 'bar', 'sub_branch': {'name': 'baz', 'sub_branch': None}},
        },
    }

    strict_validator = SchemaValidator(schema(strict=True))
    output2 = strict_validator.validate_python(input_data_strict())
    assert output_dict == output2.__dict__

    model = pydantic_model()
    if model is None:
        print('pydantic is not installed, skipping pydantic tests')
        return

    output_pydantic = model.parse_obj(input_data_lax())
    assert output_pydantic.dict() == output_dict


@pytest.mark.benchmark(group='complete')
def test_complete_core_lax(benchmark):
    v = SchemaValidator(schema())
    benchmark(v.validate_python, input_data_lax())


@pytest.mark.benchmark(group='complete')
def test_complete_core_strict(benchmark):
    v = SchemaValidator(schema(strict=True))
    benchmark(v.validate_python, input_data_strict())


@skip_pydantic
@pytest.mark.benchmark(group='complete')
def test_complete_pyd(benchmark):
    model = pydantic_model()
    assert model is not None
    benchmark(model.parse_obj, input_data_lax())
