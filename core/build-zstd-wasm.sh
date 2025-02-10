#!/usr/bin/env sh
set -eu

# clone the zstd-wasm repository
git clone --branch master --single-branch https://github.com/bokuweb/zstd-wasm
zstd_dir=$PWD/zstd-wasm
trap "rm -rf $zstd_dir" EXIT

# build the wasm and javascript files
cd $zstd_dir
git reset --hard b72295b62f73809657b1f6071601114a56b30d55
git submodule update --init --recursive
npm install

# The provided build script relies on docker, might not be suitable for all environments.
USE_DOCKER=1
if [ "$USE_DOCKER" = "1" ]; then
  npm run build:all
else
  emscripten build of zstd
  rm -rf ./zstd
  git clone https://github.com/facebook/zstd.git zstd
  cd zstd
  latest=`git describe --tags $(git rev-list --tags --max-count=1)`
  echo $latest
  git checkout -b $latest
  cd build/single_file_libs
  bash ./combine.sh -r ../../lib -o ../../../zstd.c ./zstd-in.c
  bash ./create_single_file_library.sh
  cp zstd.c ../../../zstd.c
  cd ../../../
  EM_CACHE="/tmp/emcc" emcc zstd.c -flto -o ./zstd.js -Oz --post-js export_module.js -s EXPORTED_FUNCTIONS="['_ZSTD_isError', '_ZSTD_getFrameContentSize', '_ZSTD_decompress', '_ZSTD_compress', '_ZSTD_compress_usingDict', '_ZSTD_decompress_usingDict', '_ZSTD_compressBound', '_malloc', '_free', '_ZSTD_createCCtx', '_ZSTD_createDCtx', '_ZSTD_freeCCtx', '_ZSTD_freeDCtx']" -s FILESYSTEM=0 -s ALLOW_MEMORY_GROWTH=1
  cp zstd.wasm lib/zstd.wasm
fi

# bundle the javascript file into a single loadable bundle that defines the zstd global object
npm install --dev rollup @rollup/plugin-node-resolve
node_modules/.bin/rollup ./dist/web/index.web.js --file zstd-bundle.js --format umd --name zstd --plugin @rollup/plugin-node-resolve

cp zstd-bundle.js ../zstd-bundle.js
cp zstd.wasm ../zstd.wasm
