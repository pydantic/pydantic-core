import ast
from typing import List
from dataclasses import dataclass

from test_transformer.schema_collector import SchemaValidatorCall, extract_validators_from_file
from test_transformer.core_schema_gen import convert_schema, config_to_CoreConfig, ASTClass


def insert_validator_conversions(file_path: str, validators: List[SchemaValidatorCall]) -> str:
    """
    Insert converted schema validators into the original file content.
    Returns the modified file content as a string.
    """
    # Read the original file
    with open(file_path, 'r') as f:
        content = f.read()

    # Split content into lines for easier manipulation
    lines = content.splitlines()

    # Sort validators by line number in reverse order to insert from bottom up
    # This prevents line numbers from shifting as we insert
    validators = sorted(validators, key=lambda x: x.lines.start, reverse=True)

    # Process each validator
    for validator in validators:
        # Generate the new schema code
        schema = convert_schema(validator.schema)
        config = config_to_CoreConfig(validator.config) if validator.config else None
        new_schema = (
            f'{validator.assigned_to} = SchemaValidator(schema={schema}, config={config})'
            if config
            else f'{validator.assigned_to} = SchemaValidator(schema={schema})'
        )
        # assertion = f"assert {validator.assigned_to}_n == {validator.assigned_to}"

        # Calculate indentation from the original line
        original_line = lines[validator.lines.start - 1]
        indentation = len(original_line) - len(original_line.lstrip())
        indent = ' ' * indentation

        # Format the new code with proper indentation
        new_code = f'\n{indent}# Converted schema\n{indent}{new_schema}\n'

        # Find the end of the original validator declaration
        end_line = validator.lines.end if validator.lines.end else validator.lines.start

        # Insert the new code after the original validator
        lines.insert(end_line, new_code)

    # Join lines back together
    modified_content = '\n'.join(lines)

    # add import statements
    modified_content = 'from pydantic_core import core_schema as cs\n' + modified_content

    return modified_content


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
