import gc
import platform
import re
import weakref
from typing import Any, Dict, Mapping, Union

import pytest

from pydantic_core import CoreConfig, SchemaValidator, ValidationError, core_schema

from ..conftest import Err, PyAndJson


class Cls:
    def __init__(self, **attributes):
        for k, v in attributes.items():
            setattr(self, k, v)

    def __repr__(self):
        return 'Cls({})'.format(', '.join(f'{k}={v!r}' for k, v in self.__dict__.items()))


class Map(Mapping):
    def __init__(self, **kwargs):
        self._d = kwargs

    def __iter__(self):
        return iter(self._d)

    def __len__(self) -> int:
        return len(self._d)

    def __getitem__(self, __k):
        return self._d[__k]

    def __repr__(self):
        return 'Map({})'.format(', '.join(f'{k}={v!r}' for k, v in self._d.items()))


@pytest.mark.parametrize('recursive', [False, True])
def test_simple(recursive: bool):
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'fields': {
                'field_a': {'type': 'typed-dict-field', 'schema': {'type': 'str'}},
                'field_b': {'type': 'typed-dict-field', 'schema': {'type': 'int'}},
            },
        }
    )

    assert v.construct_python({'field_a': b'abc', 'field_b': 1}, recursive=recursive) == {
        'field_a': b'abc',
        'field_b': 1,
    }


@pytest.mark.parametrize('recursive', [False, True])
def test_strict(recursive: bool):
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'fields': {
                'field_a': {'type': 'typed-dict-field', 'schema': {'type': 'str'}},
                'field_b': {'type': 'typed-dict-field', 'schema': {'type': 'int'}},
            },
            'config': {'strict': True},
        }
    )

    assert v.construct_python({'field_a': 'hello', 'field_b': 12}, recursive=recursive) == {
        'field_a': 'hello',
        'field_b': 12,
    }
    assert v.construct_python({'field_a': 123, 'field_b': '123'}, recursive=recursive) == {
        'field_a': 123,
        'field_b': '123',
    }


def test_with_default():
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'fields': {
                'field_a': {'type': 'typed-dict-field', 'schema': {'type': 'str'}},
                'field_b': {
                    'type': 'typed-dict-field',
                    'schema': {'type': 'default', 'schema': {'type': 'int'}, 'default': 666},
                },
            },
        }
    )

    assert v.construct_python({'field_a': b'abc'}, recursive=False) == {'field_a': b'abc'}
    assert v.construct_python({'field_a': b'abc'}, recursive=True) == {'field_a': b'abc', 'field_b': 666}
    assert v.construct_python({'field_a': b'abc', 'field_b': 1}, recursive=False) == {'field_a': b'abc', 'field_b': 1}
    assert v.construct_python({'field_a': b'abc', 'field_b': 1}, recursive=True) == {'field_a': b'abc', 'field_b': 1}


@pytest.mark.parametrize('recursive', [False, True])
def test_no_missing_error(recursive: bool):
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'fields': {
                'field_a': {'type': 'typed-dict-field', 'schema': {'type': 'str'}},
                'field_b': {'type': 'typed-dict-field', 'schema': {'type': 'int'}},
            },
        }
    )
    assert v.construct_python({'field_a': b'abc'}, recursive=recursive) == {'field_a': b'abc'}


@pytest.mark.parametrize('recursive', [False, True])
@pytest.mark.parametrize(
    'config,input_value,expected',
    [
        ({}, {'a': '123'}, {'a': '123'}),
        ({}, Map(a=123), {'a': 123}),  # TODO: think about
        ({}, {b'a': '123'}, {b'a': '123'}),
        ({}, {'a': '123', 'c': 4}, {'a': '123', 'c': 4}),
        ({'extra_fields_behavior': 'allow'}, {'a': '123', 'c': 4}, {'a': '123', 'c': 4}),
        ({'extra_fields_behavior': 'allow'}, {'a': '123', b'c': 4}, {'a': '123', b'c': 4}),
        ({'strict': True}, Map(a=123), {'a': 123}),  # TODO: think about
        ({}, {'a': '123', 'b': '4.7'}, {'a': '123', 'b': '4.7'}),
        ({}, {'a': '123', 'b': 'nan'}, {'a': '123', 'b': 'nan'}),
        ({'allow_inf_nan': False}, {'a': '123', 'b': 'nan'}, {'a': '123', 'b': 'nan'}),
    ],
    ids=repr,
)
def test_config(config: CoreConfig, input_value, expected, recursive):
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'fields': {
                'a': {'type': 'typed-dict-field', 'schema': {'type': 'int'}},
                'b': {'type': 'typed-dict-field', 'schema': {'type': 'float'}, 'required': False},
            },
            'config': config,
        }
    )
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            val = v.construct_python(input_value, recursive=recursive)
            print(f'UNEXPECTED OUTPUT: {val!r}')
    else:
        output_dict = v.construct_python(input_value, recursive=recursive)
        assert output_dict == expected


@pytest.mark.parametrize('recursive', [False, True])
def test_ignore_extra(recursive: bool):
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'fields': {
                'field_a': {'type': 'typed-dict-field', 'schema': {'type': 'str'}},
                'field_b': {'type': 'typed-dict-field', 'schema': {'type': 'int'}},
            },
        }
    )

    assert v.construct_python({'field_a': b'123', 'field_b': 1, 'field_c': 123}, recursive=recursive) == {
        'field_a': b'123',
        'field_b': 1,
        'field_c': 123,
    }


@pytest.mark.parametrize('recursive', [False, True])
def test_forbid_extra(recursive: bool):
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'fields': {'field_a': {'type': 'typed-dict-field', 'schema': {'type': 'str'}}},
            'extra_behavior': 'forbid',
        }
    )

    assert v.construct_python({'field_a': 'abc', 'field_b': 1}, recursive=recursive) == {'field_a': 'abc', 'field_b': 1}


@pytest.mark.parametrize('recursive', [False, True])
def test_str_config(recursive: bool):
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'fields': {'field_a': {'type': 'typed-dict-field', 'schema': {'type': 'str'}}},
            'config': {'str_max_length': 5},
        }
    )
    assert v.construct_python({'field_a': 'test'}, recursive=recursive) == {'field_a': 'test'}
    assert v.construct_python({'field_a': 'test long'}, recursive=recursive) == {'field_a': 'test long'}


@pytest.mark.parametrize('recursive', [False, True])
def test_json_error(recursive: bool):
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'fields': {
                'field_a': {'type': 'typed-dict-field', 'schema': {'type': 'list', 'items_schema': {'type': 'int'}}}
            },
        }
    )

    assert v.construct_json('{"field_a": [123, "wrong"]}', recursive=recursive) == {'field_a': [123, 'wrong']}


@pytest.mark.parametrize('recursive', [False, True])
def test_fields_required_by_default(recursive: bool):
    """By default all fields should be required, but doesn't matter when constructing"""
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'fields': {
                'x': {'type': 'typed-dict-field', 'schema': {'type': 'str'}},
                'y': {'type': 'typed-dict-field', 'schema': {'type': 'str'}},
            },
        }
    )

    assert v.construct_python({'x': 'pika', 'y': 'chu'}, recursive=recursive) == {'x': 'pika', 'y': 'chu'}
    assert v.construct_python({'x': 'pika'}, recursive=recursive) == {'x': 'pika'}


@pytest.mark.parametrize('recursive', [False, True])
def test_fields_required_by_default_with_optional(recursive: bool):
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'fields': {
                'x': {'type': 'typed-dict-field', 'schema': {'type': 'str'}},
                'y': {'type': 'typed-dict-field', 'schema': {'type': 'str'}, 'required': False},
            },
        }
    )

    assert v.construct_python({'x': 'pika', 'y': 'chu'}, recursive=recursive) == {'x': 'pika', 'y': 'chu'}
    assert v.construct_python({'x': 'pika'}, recursive=recursive) == {'x': 'pika'}


def test_fields_required_by_default_with_default():
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'fields': {
                'x': {'type': 'typed-dict-field', 'schema': {'type': 'str'}},
                'y': {
                    'type': 'typed-dict-field',
                    'schema': {'type': 'default', 'schema': {'type': 'str'}, 'default': 'bulbi'},
                },
            },
        }
    )

    assert v.construct_python({'x': 'pika', 'y': 'chu'}, recursive=False) == {'x': 'pika', 'y': 'chu'}
    assert v.construct_python({'x': 'pika', 'y': 'chu'}, recursive=True) == {'x': 'pika', 'y': 'chu'}
    assert v.construct_python({'x': 'pika'}, recursive=False) == {'x': 'pika'}
    assert v.construct_python({'x': 'pika'}, recursive=True) == {'x': 'pika', 'y': 'bulbi'}


@pytest.mark.parametrize('recursive', [False, True])
def test_all_optional_fields(recursive: bool):
    """By default all fields should be optional if `total` is set to `False`"""
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'total': False,
            'fields': {
                'x': {'type': 'typed-dict-field', 'schema': {'type': 'str', 'strict': True}},
                'y': {'type': 'typed-dict-field', 'schema': {'type': 'str'}},
            },
        }
    )

    assert v.construct_python({'x': 'pika', 'y': 'chu'}, recursive=recursive) == {'x': 'pika', 'y': 'chu'}
    assert v.construct_python({'x': 'pika'}, recursive=recursive) == {'x': 'pika'}
    assert v.construct_python({'y': 'chu'}, recursive=recursive) == {'y': 'chu'}
    assert v.construct_python({'x': 123}, recursive=recursive) == {'x': 123}


@pytest.mark.parametrize('recursive', [False, True])
def test_all_optional_fields_with_required_fields(recursive: bool):
    """Required fields not needed when constructing"""
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'total': False,
            'fields': {
                'x': {'type': 'typed-dict-field', 'schema': {'type': 'str', 'strict': True}, 'required': True},
                'y': {'type': 'typed-dict-field', 'schema': {'type': 'str'}},
            },
        }
    )

    assert v.construct_python({'x': 'pika', 'y': 'chu'}, recursive=recursive) == {'x': 'pika', 'y': 'chu'}
    assert v.construct_python({'x': 'pika'}, recursive=recursive) == {'x': 'pika'}
    assert v.construct_python({'y': 'chu'}, recursive=recursive) == {'y': 'chu'}


@pytest.mark.parametrize('recursive', [False, True])
def test_alias(py_and_json: PyAndJson, recursive: bool):
    v = py_and_json(
        {
            'type': 'typed-dict',
            'fields': {
                'field_a': {'validation_alias': 'FieldA', 'type': 'typed-dict-field', 'schema': {'type': 'int'}}
            },
        }
    )
    assert v.construct_test({'FieldA': '123'}, recursive=recursive) == {'field_a': '123'}
    assert v.construct_test({'foobar': '123'}, recursive=recursive) == {'foobar': '123'}
    assert v.construct_test({'field_a': '123'}, recursive=recursive) == {'field_a': '123'}


def test_empty_string_field_name(py_and_json: PyAndJson):
    v = py_and_json({'type': 'typed-dict', 'fields': {'': {'type': 'typed-dict-field', 'schema': {'type': 'int'}}}})
    assert v.construct_test({'': 123}) == {'': 123}


def test_empty_string_aliases(py_and_json: PyAndJson):
    v = py_and_json(
        {
            'type': 'typed-dict',
            'fields': {'field_a': {'validation_alias': '', 'type': 'typed-dict-field', 'schema': {'type': 'int'}}},
        }
    )
    assert v.construct_test({'': 123}) == {'field_a': 123}

    v = py_and_json(
        {
            'type': 'typed-dict',
            'fields': {
                'field_a': {'validation_alias': ['', ''], 'type': 'typed-dict-field', 'schema': {'type': 'int'}}
            },
        }
    )
    assert v.construct_test({'': {'': 123}}) == {'field_a': 123}


@pytest.mark.parametrize('recursive', [False, True])
def test_alias_allow_pop(py_and_json: PyAndJson, recursive: bool):
    v = py_and_json(
        {
            'type': 'typed-dict',
            'populate_by_name': True,
            'fields': {
                'field_a': {'validation_alias': 'FieldA', 'type': 'typed-dict-field', 'schema': {'type': 'int'}}
            },
        }
    )
    assert v.construct_test({'FieldA': '123'}, recursive=recursive) == {'field_a': '123'}
    assert v.construct_test({'field_a': '123'}, recursive=recursive) == {'field_a': '123'}
    assert v.construct_test({'FieldA': '1', 'field_a': '2'}, recursive=recursive) == {
        'field_a': '2'
    }  # TODO: investigate this
    assert v.construct_test({'foobar': '123'}, recursive=recursive) == {'foobar': '123'}


@pytest.mark.parametrize('recursive', [False, True])
@pytest.mark.parametrize(
    'input_value,expected',
    [
        ({'foo': {'bar': '123'}}, {'field_a': '123'}),
        # Errors finding path just return the original object
        ({'x': '123'}, {'x': '123'}),
        ({'foo': '123'}, {'foo': '123'}),
        ({'foo': [1, 2, 3]}, {'foo': [1, 2, 3]}),
        ({'foo': {'bat': '123'}}, {'foo': {'bat': '123'}}),
    ],
    ids=repr,
)
def test_alias_path(py_and_json: PyAndJson, input_value, expected, recursive):
    v = py_and_json(
        {
            'type': 'typed-dict',
            'fields': {
                'field_a': {'validation_alias': ['foo', 'bar'], 'type': 'typed-dict-field', 'schema': {'type': 'int'}}
            },
        }
    )
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=expected.message):
            v.construct_test(input_value, recursive=recursive)
    else:
        output = v.construct_test(input_value, recursive=recursive)
        assert output == expected


@pytest.mark.parametrize('recursive', [False, True])
@pytest.mark.parametrize(
    'input_value,expected',
    [
        ({'foo': {'bar': {'bat': '123'}}}, {'field_a': '123'}),
        ({'foo': [1, 2, 3, 4]}, {'field_a': 4}),
        ({'foo': (1, 2, 3, 4)}, {'field_a': 4}),
        ({'spam': 5}, {'field_a': 5}),
        ({'spam': 1, 'foo': {'bar': {'bat': 2}}}, {'spam': 1, 'field_a': 2}),
        # Errors finding path just return the original object
        ({'foo': {'x': 2}}, {'foo': {'x': 2}}),
        ({'x': '123'}, {'x': '123'}),
        ({'x': {2: 33}}, {'x': {2: 33}}),
        ({'foo': '01234'}, {'foo': '01234'}),
        ({'foo': [1]}, {'foo': [1]}),
    ],
    ids=repr,
)
def test_aliases_path_multiple(input_value, expected, recursive):
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'fields': {
                'field_a': {
                    'validation_alias': [['foo', 'bar', 'bat'], ['foo', 3], ['spam']],
                    'type': 'typed-dict-field',
                    'schema': {'type': 'int'},
                }
            },
            'config': {'loc_by_alias': False},
        }
    )
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=expected.message):
            val = v.construct_python(input_value, recursive=recursive)
            print(f'UNEXPECTED OUTPUT: {val!r}')
    else:
        output = v.construct_python(input_value, recursive=recursive)
        assert output == expected


@pytest.mark.parametrize('recursive', [False, True])
@pytest.mark.parametrize(
    'input_value,expected',
    [
        ({'foo': {-2: '123'}}, {'field_a': '123'}),
        # negatives indexes work fine
        ({'foo': [1, 42, 'xx']}, {'field_a': 42}),
        ({'foo': [42, 'xxx', 42]}, {'field_a': 'xxx'}),
        # Errors finding path just return the original object
        ({'foo': [42]}, {'foo': [42]}),
        ({'foo': {'xx': '123'}}, {'foo': {'xx': '123'}}),
        ({'foo': {'-2': '123'}}, {'foo': {'-2': '123'}}),
        ({'foo': {2: '123'}}, {'foo': {2: '123'}}),
        ({'foo': 'foobar'}, {'foo': 'foobar'}),
        ({'foo': {0, 1, 2}}, {'foo': {0, 1, 2}}),
    ],
    ids=repr,
)
def test_aliases_path_negative(input_value, expected, recursive):
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'fields': {
                'field_a': {'validation_alias': ['foo', -2], 'type': 'typed-dict-field', 'schema': {'type': 'int'}}
            },
            'config': {'loc_by_alias': False},
        }
    )
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=expected.message):
            val = v.construct_python(input_value, recursive=recursive)
            print(f'UNEXPECTED OUTPUT: {val!r}')
    else:
        output = v.construct_python(input_value, recursive=recursive)
        assert output == expected


@pytest.mark.parametrize('recursive', [False, True])
@pytest.mark.parametrize(
    'input_value,expected',
    [
        ({'foo': [1, 42, 'xx']}, {'field_a': 42}),
        ({'foo': [42, 'xxx', 42]}, {'field_a': 'xxx'}),
        ({'foo': [42]}, {'foo': [42]}),  # Errors finding path just return the original object
    ],
    ids=repr,
)
def test_aliases_path_negative_json(py_and_json: PyAndJson, input_value, expected, recursive):
    v = py_and_json(
        {
            'type': 'typed-dict',
            'fields': {
                'field_a': {'validation_alias': ['foo', -2], 'type': 'typed-dict-field', 'schema': {'type': 'int'}}
            },
        }
    )
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=expected.message):
            val = v.construct_test(input_value, recursive=recursive)
            print(f'UNEXPECTED OUTPUT: {val!r}')
    else:
        output = v.construct_test(input_value, recursive=recursive)
        assert output == expected


@pytest.mark.parametrize('recursive', [False, True])
def test_alias_error_loc_alias(recursive: bool):
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'fields': {
                'field_a': {
                    'type': 'typed-dict-field',
                    'schema': {'type': 'int'},
                    'validation_alias': [['foo', 'x'], ['bar', 1, -1]],
                }
            },
        },
        {'loc_by_alias': True},  # this is the default
    )
    assert v.construct_python({'foo': {'x': 42}}, recursive=recursive) == {'field_a': 42}
    assert v.construct_python({'bar': ['x', {-1: 42}]}, recursive=recursive) == {'field_a': 42}
    assert v.construct_python({'bar': ['x', [1, 2, 42]]}, recursive=recursive) == {'field_a': 42}
    assert v.construct_python({'foo': {'x': 'not_int'}}, recursive=recursive) == {'field_a': 'not_int'}
    assert v.construct_python({'bar': ['x', [1, 2, 'not_int']]}, recursive=recursive) == {'field_a': 'not_int'}
    assert v.construct_python({}, recursive=recursive) == {}


@pytest.mark.parametrize('recursive', [False, True])
def test_alias_error_loc_field_names(py_and_json: PyAndJson, recursive: bool):
    v = py_and_json(
        {
            'type': 'typed-dict',
            'fields': {
                'field_a': {
                    'type': 'typed-dict-field',
                    'schema': {'type': 'int'},
                    'validation_alias': [['foo'], ['bar', 1, -1]],
                }
            },
            'config': {'loc_by_alias': False},
        }
    )
    assert v.construct_test({'foo': 42}, recursive=recursive) == {'field_a': 42}
    assert v.construct_test({'bar': ['x', [1, 2, 42]]}, recursive=recursive) == {'field_a': 42}
    assert v.construct_test({'foo': 'not_int'}, recursive=recursive) == {'field_a': 'not_int'}
    assert v.construct_test({'bar': ['x', [1, 2, 'not_int']]}, recursive=recursive) == {'field_a': 'not_int'}
    assert v.construct_test({}, recursive=recursive) == {}


@pytest.mark.parametrize('recursive', [False, True])
def test_empty_model(recursive: bool):
    v = SchemaValidator({'type': 'typed-dict', 'fields': {}})
    assert v.construct_python({}, recursive=recursive) == {}
    assert v.construct_python('x', recursive=recursive) == 'x'


@pytest.mark.parametrize('recursive', [False, True])
def test_model_deep(recursive: bool):
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'fields': {
                'field_a': {'type': 'typed-dict-field', 'schema': {'type': 'str'}},
                'field_b': {
                    'type': 'typed-dict-field',
                    'schema': {
                        'type': 'typed-dict',
                        'fields': {
                            'field_c': {'type': 'typed-dict-field', 'schema': {'type': 'str'}},
                            'field_d': {
                                'type': 'typed-dict-field',
                                'schema': {
                                    'type': 'typed-dict',
                                    'fields': {
                                        'field_e': {'type': 'typed-dict-field', 'schema': {'type': 'str'}},
                                        'field_f': {'type': 'typed-dict-field', 'schema': {'type': 'int'}},
                                    },
                                },
                            },
                        },
                    },
                },
            },
        }
    )
    output = v.construct_python(
        {'field_a': '1', 'field_b': {'field_c': '2', 'field_d': {'field_e': '4', 'field_f': 4}}}, recursive=recursive
    )
    assert output == {'field_a': '1', 'field_b': ({'field_c': '2', 'field_d': {'field_e': '4', 'field_f': 4}})}
    output = v.construct_python(
        {'field_a': '1', 'field_b': {'field_c': '2', 'field_d': {'field_e': '4', 'field_f': 'xx'}}}, recursive=recursive
    )
    assert output == {'field_a': '1', 'field_b': {'field_c': '2', 'field_d': {'field_e': '4', 'field_f': 'xx'}}}


@pytest.mark.parametrize('recursive', [False, True])
def test_alias_extra(py_and_json: PyAndJson, recursive: bool):
    v = py_and_json(
        {
            'type': 'typed-dict',
            'extra_behavior': 'allow',
            'fields': {
                'field_a': {
                    'validation_alias': [['FieldA'], ['foo', 2]],
                    'type': 'typed-dict-field',
                    'schema': {'type': 'int'},
                }
            },
            'config': {'loc_by_alias': False},
        }
    )
    assert v.construct_test({'FieldA': 1}, recursive=recursive) == {'field_a': 1}
    assert v.construct_test({'foo': [1, 2, 3]}, recursive=recursive) == {'field_a': 3}
    assert v.construct_test({'FieldA': '...'}, recursive=recursive) == {'field_a': '...'}


@pytest.mark.parametrize('recursive', [False, True])
def test_alias_extra_by_name(py_and_json: PyAndJson, recursive: bool):
    v = py_and_json(
        {
            'type': 'typed-dict',
            'extra_behavior': 'allow',
            'populate_by_name': True,
            'fields': {
                'field_a': {'validation_alias': 'FieldA', 'type': 'typed-dict-field', 'schema': {'type': 'int'}}
            },
        }
    )
    assert v.construct_test({'FieldA': 1}, recursive=recursive) == {'field_a': 1}
    assert v.construct_test({'field_a': 1}, recursive=recursive) == {'field_a': 1}


@pytest.mark.parametrize('recursive', [False, True])
def test_alias_extra_forbid(py_and_json: PyAndJson, recursive: bool):
    v = py_and_json(
        {
            'type': 'typed-dict',
            'extra_behavior': 'forbid',
            'fields': {
                'field_a': {'type': 'typed-dict-field', 'validation_alias': 'FieldA', 'schema': {'type': 'int'}}
            },
        }
    )
    assert v.construct_test({'FieldA': 1}, recursive=recursive) == {'field_a': 1}


def test_with_default_factory():
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'fields': {
                'x': {
                    'type': 'typed-dict-field',
                    'schema': {'type': 'default', 'schema': {'type': 'str'}, 'default_factory': lambda: 'pikachu'},
                }
            },
        }
    )

    assert v.construct_python({}, recursive=False) == {}
    assert v.construct_python({}, recursive=True) == {'x': 'pikachu'}
    assert v.construct_python({'x': 'bulbi'}, recursive=False) == {'x': 'bulbi'}
    assert v.construct_python({'x': 'bulbi'}, recursive=True) == {'x': 'bulbi'}


@pytest.mark.parametrize(
    'default_factory,error_message',
    [
        (lambda: 1 + 'a', "unsupported operand type(s) for +: 'int' and 'str'"),
        (lambda x: 'a' + x, "<lambda>() missing 1 required positional argument: 'x'"),
    ],
)
def test_bad_default_factory(default_factory, error_message):
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'fields': {
                'x': {
                    'type': 'typed-dict-field',
                    'schema': {'type': 'default', 'schema': {'type': 'str'}, 'default_factory': default_factory},
                }
            },
        }
    )
    with pytest.raises(TypeError, match=re.escape(error_message)):
        v.construct_python({}, recursive=True)


@pytest.mark.parametrize('recursive', [False, True])
class TestOnError:
    def test_on_error_raise_by_default(self, py_and_json: PyAndJson, recursive: bool):
        """Invalid values don't create errors"""
        v = py_and_json(
            {'type': 'typed-dict', 'fields': {'x': {'type': 'typed-dict-field', 'schema': {'type': 'str'}}}}
        )
        assert v.construct_test({'x': 'foo'}, recursive=recursive) == {'x': 'foo'}
        assert v.construct_test({'x': ['foo']}, recursive=recursive) == {'x': ['foo']}

    def test_on_error_raise_explicit(self, py_and_json: PyAndJson, recursive: bool):
        """Invalid values don't create errors"""
        v = py_and_json(
            {
                'type': 'typed-dict',
                'fields': {
                    'x': {
                        'type': 'typed-dict-field',
                        'schema': {'type': 'default', 'schema': {'type': 'str'}, 'on_error': 'raise'},
                    }
                },
            }
        )
        assert v.construct_test({'x': 'foo'}, recursive=recursive) == {'x': 'foo'}
        assert v.construct_test({'x': ['foo']}, recursive=recursive) == {'x': ['foo']}

    def test_on_error_omit(self, py_and_json: PyAndJson, recursive: bool):
        """Values are not omitted since no error is generated"""
        v = py_and_json(
            {
                'type': 'typed-dict',
                'fields': {
                    'x': {
                        'type': 'typed-dict-field',
                        'schema': {'type': 'default', 'schema': {'type': 'str'}, 'on_error': 'omit'},
                        'required': False,
                    }
                },
            }
        )
        assert v.construct_test({'x': 'foo'}, recursive=recursive) == {'x': 'foo'}
        assert v.construct_test({}, recursive=recursive) == {}
        assert v.construct_test({'x': ['foo']}, recursive=recursive) == {'x': ['foo']}

    def test_on_error_omit_with_default(self, py_and_json: PyAndJson, recursive: bool):
        v = py_and_json(
            {
                'type': 'typed-dict',
                'fields': {
                    'x': {
                        'type': 'typed-dict-field',
                        'schema': {'type': 'default', 'schema': {'type': 'str'}, 'on_error': 'omit', 'default': 'pika'},
                        'required': False,
                    }
                },
            }
        )
        assert v.construct_test({'x': 'foo'}, recursive=recursive) == {'x': 'foo'}
        assert v.construct_test({}, recursive=False) == {}
        assert v.construct_test({}, recursive=True) == {'x': 'pika'}
        assert v.construct_test({'x': ['foo']}, recursive=recursive) == {'x': ['foo']}

    def test_on_error_default(self, py_and_json: PyAndJson, recursive: bool):
        """Defaults are not used since no error is generated"""
        v = py_and_json(
            {
                'type': 'typed-dict',
                'fields': {
                    'x': {
                        'type': 'typed-dict-field',
                        'schema': {
                            'type': 'default',
                            'schema': {'type': 'str'},
                            'on_error': 'default',
                            'default': 'pika',
                        },
                    }
                },
            }
        )
        assert v.construct_test({'x': 'foo'}, recursive=recursive) == {'x': 'foo'}
        assert v.construct_test({'x': ['foo']}, recursive=recursive) == {'x': ['foo']}

    def test_on_error_default_factory(self, py_and_json: PyAndJson, recursive: bool):
        """Default factory functions are not called"""
        v = py_and_json(
            {
                'type': 'typed-dict',
                'fields': {
                    'x': {
                        'type': 'typed-dict-field',
                        'schema': {
                            'type': 'default',
                            'schema': {'type': 'str'},
                            'on_error': 'default',
                            'default_factory': lambda: 'pika',
                        },
                    }
                },
            }
        )
        assert v.construct_test({'x': 'foo'}, recursive=recursive) == {'x': 'foo'}
        assert v.construct_test({'x': ['foo']}, recursive=recursive) == {'x': ['foo']}

    def test_wrap_on_error(self, py_and_json: PyAndJson, recursive: bool):
        """Validation functions are not called"""

        def wrap_function(input_value, validator, info):
            try:
                return validator(input_value)
            except ValidationError:
                if isinstance(input_value, list):
                    return str(len(input_value))
                else:
                    return repr(input_value)

        v = py_and_json(
            {
                'type': 'typed-dict',
                'fields': {
                    'x': {
                        'type': 'typed-dict-field',
                        'schema': {
                            'type': 'default',
                            'on_error': 'raise',
                            'schema': {
                                'type': 'function-wrap',
                                'function': {'type': 'general', 'function': wrap_function},
                                'schema': {'type': 'str'},
                            },
                        },
                    }
                },
            }
        )
        assert v.construct_test({'x': 'foo'}, recursive=recursive) == {'x': 'foo'}
        assert v.construct_test({'x': ['foo']}, recursive=recursive) == {'x': ['foo']}
        assert v.construct_test({'x': ['foo', 'bar']}, recursive=recursive) == {'x': ['foo', 'bar']}
        assert v.construct_test({'x': {'a': 'b'}}, recursive=recursive) == {'x': {'a': 'b'}}


@pytest.mark.parametrize('recursive', [False, True])
@pytest.mark.parametrize(
    'config,schema_extra_behavior_kw',
    [
        (core_schema.CoreConfig(extra_fields_behavior='allow'), {}),
        (core_schema.CoreConfig(extra_fields_behavior='allow'), {'extra_behavior': None}),
        (core_schema.CoreConfig(), {'extra_behavior': 'allow'}),
        (None, {'extra_behavior': 'allow'}),
        (core_schema.CoreConfig(extra_fields_behavior='forbid'), {'extra_behavior': 'allow'}),
    ],
)
@pytest.mark.parametrize(
    'extras_schema_kw, expected_extra_value',
    [({}, '123'), ({'extras_schema': None}, '123'), ({'extras_schema': core_schema.int_schema()}, '123')],
    ids=['extras_schema=unset', 'extras_schema=None', 'extras_schema=int'],
)
def test_extra_behavior_allow(
    config: Union[core_schema.CoreConfig, None],
    schema_extra_behavior_kw: Dict[str, Any],
    extras_schema_kw: Dict[str, Any],
    expected_extra_value: Any,
    recursive: bool,
):
    v = SchemaValidator(
        core_schema.typed_dict_schema(
            {'f': core_schema.typed_dict_field(core_schema.str_schema())},
            **schema_extra_behavior_kw,
            **extras_schema_kw,
            config=config,
        )
    )

    m: Dict[str, Any] = v.construct_python({'f': 'x', 'extra_field': '123'}, recursive=recursive)
    assert m == {'f': 'x', 'extra_field': expected_extra_value}


@pytest.mark.parametrize('recursive', [False, True])
@pytest.mark.parametrize(
    'config,schema_extra_behavior_kw',
    [
        (core_schema.CoreConfig(extra_fields_behavior='forbid'), {}),
        (core_schema.CoreConfig(extra_fields_behavior='forbid'), {'extra_behavior': None}),
        (core_schema.CoreConfig(), {'extra_behavior': 'forbid'}),
        (None, {'extra_behavior': 'forbid'}),
        (core_schema.CoreConfig(extra_fields_behavior='allow'), {'extra_behavior': 'forbid'}),
    ],
)
def test_extra_behavior_forbid(
    config: Union[core_schema.CoreConfig, None], schema_extra_behavior_kw: Dict[str, Any], recursive: bool
):
    """forbid has no effect when constructing"""
    v = SchemaValidator(
        core_schema.typed_dict_schema(
            {'f': core_schema.typed_dict_field(core_schema.str_schema())}, **schema_extra_behavior_kw, config=config
        )
    )

    m: Dict[str, Any] = v.construct_python({'f': 'x'}, recursive=recursive)
    assert m == {'f': 'x'}
    assert v.construct_python({'f': 'x', 'extra_field': 123}, recursive=recursive) == {'f': 'x', 'extra_field': 123}


@pytest.mark.parametrize(
    'config,schema_extra_behavior_kw',
    [
        (core_schema.CoreConfig(extra_fields_behavior='ignore'), {}),
        (core_schema.CoreConfig(), {'extra_behavior': 'ignore'}),
        (None, {'extra_behavior': 'ignore'}),
        (core_schema.CoreConfig(extra_fields_behavior='forbid'), {'extra_behavior': 'ignore'}),
        (core_schema.CoreConfig(), {}),
        (core_schema.CoreConfig(), {'extra_behavior': None}),
        (None, {'extra_behavior': None}),
    ],
)
def test_extra_behavior_ignore(config: Union[core_schema.CoreConfig, None], schema_extra_behavior_kw: Dict[str, Any]):
    """ignore has no effect when constructing"""
    v = SchemaValidator(
        core_schema.typed_dict_schema(
            {'f': core_schema.typed_dict_field(core_schema.str_schema())}, **schema_extra_behavior_kw
        ),
        config=config,
    )

    m: Dict[str, Any] = v.construct_python({'f': 'x', 'extra_field': 123})
    assert m == {'f': 'x', 'extra_field': 123}


@pytest.mark.xfail(
    condition=platform.python_implementation() == 'PyPy', reason='https://foss.heptapod.net/pypy/pypy/-/issues/3899'
)
def test_leak_typed_dict():
    def fn():
        def validate(v, info):
            return v

        schema = core_schema.general_plain_validator_function(validate)
        schema = core_schema.typed_dict_schema(
            {'f': core_schema.typed_dict_field(schema)}, extra_behavior='allow', extras_schema=schema
        )

        # If any of the Rust validators don't implement traversal properly,
        # there will be an undetectable cycle created by this assignment
        # which will keep Defaulted alive
        validate.__pydantic_validator__ = SchemaValidator(schema)

        return validate

    cycle = fn()
    ref = weakref.ref(cycle)
    assert ref() is not None

    del cycle
    gc.collect(0)
    gc.collect(1)
    gc.collect(2)
    gc.collect()

    assert ref() is None


def test_typed_dict_recursive():
    class Child:
        a: int
        b: int

    child_schema = core_schema.model_schema(
        Child,
        core_schema.model_fields_schema(
            {
                'a': core_schema.model_field(core_schema.int_schema()),
                'b': core_schema.model_field(core_schema.int_schema()),
            }
        ),
    )

    # TypedDict[str, Child]
    v = SchemaValidator(core_schema.typed_dict_schema({'child': core_schema.typed_dict_field(child_schema)}))

    assert v.construct_python(None) is None
    assert v.construct_python(None, recursive=True) is None

    # Test recursive
    assert v.construct_python({'child': {'a': 10, 'b': 'wrong'}}) == {'child': {'a': 10, 'b': 'wrong'}}
    m = v.construct_python({'child': {'a': 10, 'b': 'wrong'}}, recursive=True)
    assert isinstance(m['child'], Child)
    assert m['child'].a == 10
    assert m['child'].b == 'wrong'

    # If the key is the wrong name, don't recurse
    assert v.construct_python({'wrong': {'a': 10, 'b': 'wrong'}}) == {'wrong': {'a': 10, 'b': 'wrong'}}
    assert v.construct_python({'wrong': {'a': 10, 'b': 'wrong'}}, recursive=True) == {'wrong': {'a': 10, 'b': 'wrong'}}
