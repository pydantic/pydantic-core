import ast
from typing import List
from dataclasses import dataclass

from test_transformer.schema_collector import SchemaValidatorCall, extract_validators_from_file, ParametrizedConfig, ParametrizedSchema
from test_transformer.core_schema_gen import convert_schema, config_to_CoreConfig, ASTClass


def get_replace_target(source:str, lines, char_range):
    source_lines = source.splitlines()
    replace_target = []
    for i in range(lines.start - 1, lines.end):
        replace_target.append(source_lines[i])
    replace_target[-1] = replace_target[-1][:char_range.end]
    replace_target[0] = replace_target[0][char_range.start:]
    return "\n".join(replace_target)

def insert_validator_conversions(file_path: str, validators: List[SchemaValidatorCall]) -> str:
    """
    Insert converted schema validators into the original file content.
    Returns the modified file content as a string.
    """
    # Read the original file
    with open(file_path, 'r') as f:
        content = f.read()

    # Make a copy of the content to modify
    modified_content = content

    # Sort validators by line number in reverse order to insert from bottom up
    # This prevents line numbers from shifting as we insert
    validators = sorted(validators, key=lambda x: x.lines.start, reverse=True)

    # Process each validator
    for validator in validators:
        # Generate the new schema code
        if isinstance(validator, ParametrizedConfig):
            new_code = config_to_CoreConfig(validator.config) 
            
        elif isinstance(validator, ParametrizedSchema):
            new_code = convert_schema(validator.schema)

        elif isinstance(validator, SchemaValidatorCall):
            schema = convert_schema(validator.schema)
            config = config_to_CoreConfig(validator.config) if validator.config else None
            new_code = (
                f'SchemaValidator(schema={schema}, config={config})'
                if config
                else f'SchemaValidator(schema={schema})'
            )
        else:
            raise ValueError(f'Unsupported change type: {type(validator)}')

        # Find the code to replace
        replace_target = get_replace_target(content, validator.lines, validator.char_range)

        # Replace the code in the modified content
        modified_content = modified_content.replace(replace_target, new_code)


    # add import statements

    return 'from pydantic_core import core_schema as cs\n' + modified_content


def update_test_file(input_file: str, output_file: str):
    """
    Process a test file and update it with converted schema validators.
    If output_file is not specified, will modify the input file in place.
    """
    # Extract validators
    validators = extract_validators_from_file(input_file)

    # Generate modified content
    modified_content = insert_validator_conversions(input_file, validators)

    # Write to output file
    output_path = output_file or input_file
    with open(output_path, 'w') as f:
        f.write(modified_content)

    print(f'Updated {len(validators)} schema validators in {output_path}')


if __name__ == '__main__':
    # Example usage
    update_test_file('tests/test_config.py', 'tests/test_config_updated.py')
