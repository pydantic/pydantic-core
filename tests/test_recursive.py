from pydantic_core import SchemaValidator


def test_branch_optional():
    v = SchemaValidator(
        {
            'type': 'recursive-container',
            'schema': {
                'type': 'model',
                'fields': {
                    'name': {'type': 'str'},
                    'sub_branch': {
                        'type': 'union',
                        'default': None,
                        'choices': [{'type': 'none'}, {'type': 'recursive-ref'}],
                    },
                },
            },
        }
    )

    assert v.validate_python({'name': 'root', 'sub_branch': {'name': 'b1'}}) == (
        {'name': 'root', 'sub_branch': ({'name': 'b1', 'sub_branch': None}, {'name'})},
        {'sub_branch', 'name'},
    )
