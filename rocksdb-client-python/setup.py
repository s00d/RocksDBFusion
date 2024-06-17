from setuptools import setup, find_packages

setup(
    name='rocksdb-client',
    version='0.1.1',
    packages=find_packages(where='src'),
    package_dir={'': 'src'},
    install_requires=[
        'asyncio',
    ],
    test_suite='tests',
    author='s00d',
    author_email='Virus191288@gmail.com',
    description='Python client for interacting with RocksDB server',
    long_description=open('README.md').read(),
    long_description_content_type='text/markdown',
    url='https://github.com/s00d/RocksDBFusion',
    classifiers=[
        'Programming Language :: Python :: 3',
        'License :: OSI Approved :: MIT License',
        'Operating System :: OS Independent',
    ],
    python_requires='>=3.6',
)