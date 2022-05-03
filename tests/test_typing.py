from pydantic_core import Schema


class Foo:
    bar: str


def foo(bar: str) -> None:
    ...


def test_schema_typing() -> None:
    # this gets run by pyright, but we also check that it executes
    _: Schema = {
        'type': 'union',
        'choices': [
            {'type': 'int', 'ge': 1},
            {'type': 'float', 'lt': 1.0},
            {'type': 'str', 'pattern': r'http:\/\/.*'},
            {'type': 'bool', 'strict': False},
            {'type': 'literal', 'expected': [1, '1']},
            {'type': 'any'},
            {'type': 'none'},
            {'type': 'list', 'items': {'type': 'str'}, 'min_items': 3},
            {'type': 'set', 'items': {'type': 'str'}, 'max_items': 3},
            {'type': 'dict', 'keys': {'type': 'str'}, 'values': {'type': 'any'}},
            {'type': 'model-class', 'class_type': Foo, 'model': {'type': 'model', 'fields': {'bar': {'type': 'str'}}}},
            {'type': 'function', 'mode': 'wrap', 'function': foo},
            {
                'type': 'recursive-container',
                'name': 'Branch',
                'schema': {
                    'type': 'model',
                    'fields': {
                        'name': {'type': 'str'},
                        'sub_branch': {
                            'type': 'union',
                            'default': None,
                            'choices': [{'type': 'none'}, {'type': 'recursive-ref', 'name': 'Branch'}],
                        },
                    },
                },
            },
        ],
    }
