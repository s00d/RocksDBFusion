import resolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import typescript from '@rollup/plugin-typescript';
import polyfillNode from 'rollup-plugin-polyfill-node';

export default {
    input: 'src/index.ts',
    output: [
        {
            file: 'dist/index.cjs.js',
            format: 'cjs',
        },
        {
            file: 'dist/index.es.js',
            format: 'es',
        },
        {
            file: 'dist/index.browser.js',
            format: 'iife',
            name: 'RocksDBClient',
        },
    ],
    plugins: [
        resolve({
            browser: true,
        }),
        commonjs(),
        typescript({
            tsconfig: './tsconfig.json',
        }),
        polyfillNode(),
    ],
};
