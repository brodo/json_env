const {Binary} = require('binary-install');
const os = require('os');

function getPlatform() {
    const type = os.type();
    const arch = os.arch();

    if (type === 'Windows_NT' && arch === 'x64') {
        return 'x86_64-windows';
    }

    if (type === 'Linux') {
        if (arch === 'x64') {
            return 'x86_64-linux';
        }
        if (arch === 'arm64') {
            return 'aarch64-linux';
        }
    }

    if (type === 'Darwin') {
        if(arch === 'x64') {
            return 'x86_64-macos'
        }
        if(arch === 'arm64')  {
            return 'aarch64-macos';
        }
    }
    throw new Error(`Unsupported platform: ${type} ${arch}. Please create an issue at https://github.com/brodo/json_env/issues`);
}

function getBinary() {
    const platform = getPlatform();
    const version = require('../package.json').version;
    const extension = platform === 'x86_64-windows' ? 'zip' : 'tar.gz';
    const url = `https://github.com/brodo/json_env/releases/download/v${version}/json_env-v${version}-${platform}.${extension}`;
    return new Binary('json_env', url);
}

module.exports = getBinary;