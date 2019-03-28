const path = require('path');

const CopyWebpackPlugin = require('copy-webpack-plugin');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');

module.exports = {
    entry: './bootstrap.js',
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'bootstrap.js',
    },
    plugins: [
        new CopyWebpackPlugin(['index.html']),
        // The directory where 'pkg' was created by 'wasm-pack':
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, '..')
        })
    ],
    mode: 'development'
};