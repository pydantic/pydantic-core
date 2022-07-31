import os
import shutil
import sys
from pathlib import Path
from zipfile import ZipFile


def error(msg: str):
    print(f'ERROR: {msg}', file=sys.stderr)
    exit(1)


try:
    import requests
except ImportError:
    error("requests not installed, you'll need to run `pip install requests`")


def main():
    this_dir = Path(__file__).parent
    root_dir = this_dir.parent
    tests_dir = root_dir / 'tests'
    for pycache_dir in tests_dir.glob('**/__pycache__'):
        shutil.rmtree(pycache_dir)
    for pycache_dir in this_dir.glob('**/.hypothesis'):
        shutil.rmtree(pycache_dir)

    zip_file = this_dir / 'tests.zip'
    zipped_files = 0
    with ZipFile(zip_file, 'w') as zf:
        for py_file in tests_dir.glob('**/*.py'):
            zf.write(py_file, py_file.relative_to(tests_dir))
            zipped_files += 1
    print(f'Zipped {zipped_files} test files')

    try:
        wheel_file = next(p for p in (root_dir / 'dist').iterdir() if p.name.endswith('wasm32.whl'))
    except StopIteration:
        error('No wheel found in "dist" directory')
    else:
        uploader = Uploader()
        uploader.upload_file(zip_file)
        uploader.upload_file(this_dir / 'worker.js', content_type='application/javascript')
        uploader.upload_file(this_dir / 'index.html', content_type='text/html')
        wheel_url = uploader.upload_file(wheel_file)

        print('Setting "{{ pydantic_core_wheel_url }}" to "%s" in run_tests.py' % wheel_url)
        run_test_code = (this_dir / 'run_tests.py').read_text().replace('{{ pydantic_core_wheel_url }}', wheel_url)
        uploader.upload_file(run_test_code.encode(), url_path='run_tests.py', content_type='text/plain')

        print('upload complete ✓ visit', uploader.url)


class Uploader:
    def __init__(self):
        try:
            auth_key = os.environ['SMOKESHOW_AUTH_KEY']
        except KeyError:
            raise RuntimeError('No auth key provided, please set SMOKESHOW_AUTH_KEY')
        else:
            self.client = requests.Session()
            r = self.client.post('https://smokeshow.helpmanual.io/create/', headers={'Authorisation': auth_key})
            if r.status_code != 200:
                raise ValueError(f'Error creating ephemeral site {r.status_code}, response:\n{r.text}')

            obj = r.json()
            self.secret_key: str = obj['secret_key']
            self.url: str = obj['url']
            assert self.url.endswith('/'), self.url

    def upload_file(self, file: 'Path | bytes', *, url_path: str = None, content_type: str = None) -> str:
        headers = {'Authorisation': self.secret_key, 'Response-Header-Access-Control-Allow-Origin': '*'}
        if content_type:
            headers['Content-Type'] = content_type

        if isinstance(file, Path):
            content = file.read_bytes()
            url_path = url_path or file.name
        else:
            assert isinstance(file, bytes), 'file must be a Path or bytes'
            content = file
            assert url_path, 'url_path must be provided if file is bytes'

        url = self.url + url_path
        r = self.client.post(url, data=content, headers=headers)
        if r.status_code == 200:
            upload_info = r.json()
            print(f'    uploaded {url_path} size={upload_info["size"]:,}')
        else:
            print(f'    ERROR! {url_path} status={r.status_code} response={r.text}')
            error(f'invalid response from "{url_path}" status={r.status_code} response={r.text}')
        return url


if __name__ == '__main__':
    main()
