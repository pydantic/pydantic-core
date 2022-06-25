from datetime import date, datetime, time

import pytest

from pydantic_core import SchemaValidator

from .complete_schema import complete_input_data, complete_schema, complete_pydantic_model
from .test_micro_benchmarks import skip_pydantic


def test_complete_core_test():
    schema = complete_schema()
    class_type = schema['class_type']
    v = SchemaValidator(schema)
    input_data = complete_input_data()
    output = v.validate_python(input_data)
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
        'field_date': date(2020, 1, 1),
        'field_date_con': date(2020, 1, 1),
        'field_time': time(12, 0),
        'field_time_con': time(12, 0),
        'field_datetime': datetime(2020, 1, 1, 0, 0),
        'field_datetime_con': datetime(2020, 1, 1, 0, 0),
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
    pydantic_model = complete_pydantic_model()
    if pydantic_model is None:
        return

    output_pydantic = pydantic_model.parse_obj(input_data)
    assert output_pydantic.dict() == output_dict


@pytest.mark.benchmark(group='complete')
def test_complete_core(benchmark):
    v = SchemaValidator(complete_schema())
    input_data = complete_input_data()
    benchmark(v.validate_python, input_data)


@skip_pydantic
@pytest.mark.benchmark(group='complete')
def test_complete_pyd(benchmark):
    pydantic_model = complete_pydantic_model()
    assert pydantic_model is not None
    input_data = complete_input_data()
    benchmark(pydantic_model.parse_obj, input_data)
