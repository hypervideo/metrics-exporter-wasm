(function (global, factory) {
    typeof exports === 'object' && typeof module !== 'undefined' ? factory(exports) :
    typeof define === 'function' && define.amd ? define(['exports'], factory) :
    (global = typeof globalThis !== 'undefined' ? globalThis : global || self, factory(global.zstd = {}));
})(this, (function (exports) { 'use strict';

    var _documentCurrentScript = typeof document !== 'undefined' ? document.currentScript : null;
    // @ts-nocheck
    var Module = typeof Module !== 'undefined' ? Module : {};
    var moduleOverrides = {};
    var key;
    for (key in Module) {
        if (Module.hasOwnProperty(key)) {
            moduleOverrides[key] = Module[key];
        }
    }
    var err = Module['printErr'] || console.warn.bind(console);
    for (key in moduleOverrides) {
        if (moduleOverrides.hasOwnProperty(key)) {
            Module[key] = moduleOverrides[key];
        }
    }
    moduleOverrides = null;
    if (Module['arguments'])
        Module['arguments'];
    if (Module['thisProgram'])
        thisProgram = Module['thisProgram'];
    if (Module['quit'])
        quit_ = Module['quit'];
    if (typeof WebAssembly !== 'object') {
        abort('no native wasm support detected');
    }
    var wasmMemory;
    var ABORT = false;
    function alignUp(x, multiple) {
        if (x % multiple > 0) {
            x += multiple - (x % multiple);
        }
        return x;
    }
    var buffer, HEAPU8;
    function updateGlobalBufferAndViews(buf) {
        buffer = buf;
        Module['HEAP8'] = new Int8Array(buf);
        Module['HEAPU8'] = HEAPU8 = new Uint8Array(buf);
    }
    Module['INITIAL_MEMORY'] || 16777216;
    var wasmTable;
    var __ATPRERUN__ = [];
    var __ATINIT__ = [];
    var __ATPOSTRUN__ = [];
    function preRun() {
        if (Module['preRun']) {
            if (typeof Module['preRun'] == 'function')
                Module['preRun'] = [Module['preRun']];
            while (Module['preRun'].length) {
                addOnPreRun(Module['preRun'].shift());
            }
        }
        callRuntimeCallbacks(__ATPRERUN__);
    }
    function initRuntime() {
        callRuntimeCallbacks(__ATINIT__);
    }
    function postRun() {
        if (Module['postRun']) {
            if (typeof Module['postRun'] == 'function')
                Module['postRun'] = [Module['postRun']];
            while (Module['postRun'].length) {
                addOnPostRun(Module['postRun'].shift());
            }
        }
        callRuntimeCallbacks(__ATPOSTRUN__);
    }
    function addOnPreRun(cb) {
        __ATPRERUN__.unshift(cb);
    }
    function addOnInit(cb) {
        __ATINIT__.unshift(cb);
    }
    function addOnPostRun(cb) {
        __ATPOSTRUN__.unshift(cb);
    }
    var runDependencies = 0;
    var dependenciesFulfilled = null;
    function addRunDependency(id) {
        runDependencies++;
        if (Module['monitorRunDependencies']) {
            Module['monitorRunDependencies'](runDependencies);
        }
    }
    function removeRunDependency(id) {
        runDependencies--;
        if (Module['monitorRunDependencies']) {
            Module['monitorRunDependencies'](runDependencies);
        }
        if (runDependencies == 0) {
            if (dependenciesFulfilled) {
                var callback = dependenciesFulfilled;
                dependenciesFulfilled = null;
                callback();
            }
        }
    }
    Module['preloadedImages'] = {};
    Module['preloadedAudios'] = {};
    function abort(what) {
        if (Module['onAbort']) {
            Module['onAbort'](what);
        }
        what += '';
        err(what);
        ABORT = true;
        what = 'abort(' + what + ').';
        var e = new WebAssembly.RuntimeError(what);
        throw e;
    }
    function getBinaryPromise(url) {
        return fetch(url, { credentials: 'same-origin' }).then(function (response) {
            if (!response['ok']) {
                throw "failed to load wasm binary file at '" + url + "'";
            }
            return response['arrayBuffer']();
        });
    }
    function init$1(filePathOrBuf) {
        var info = { a: asmLibraryArg };
        function receiveInstance(instance, module) {
            var exports = instance.exports;
            Module['asm'] = exports;
            wasmMemory = Module['asm']['c'];
            updateGlobalBufferAndViews(wasmMemory.buffer);
            wasmTable = Module['asm']['r'];
            addOnInit(Module['asm']['d']);
            removeRunDependency();
        }
        addRunDependency();
        function receiveInstantiationResult(result) {
            receiveInstance(result['instance']);
        }
        function instantiateArrayBuffer(receiver) {
            return getBinaryPromise(filePathOrBuf)
                .then(function (binary) {
                var result = WebAssembly.instantiate(binary, info);
                return result;
            })
                .then(receiver, function (reason) {
                err('failed to asynchronously prepare wasm: ' + reason);
                abort(reason);
            });
        }
        function instantiateAsync() {
            if (filePathOrBuf && filePathOrBuf.byteLength > 0) {
                return WebAssembly.instantiate(filePathOrBuf, info).then(receiveInstantiationResult, function (reason) {
                    err('wasm compile failed: ' + reason);
                });
            }
            else if (typeof WebAssembly.instantiateStreaming === 'function' &&
                typeof filePathOrBuf === 'string' &&
                typeof fetch === 'function') {
                return fetch(filePathOrBuf, { credentials: 'same-origin' }).then(function (response) {
                    var result = WebAssembly.instantiateStreaming(response, info);
                    return result.then(receiveInstantiationResult, function (reason) {
                        err('wasm streaming compile failed: ' + reason);
                        err('falling back to ArrayBuffer instantiation');
                        return instantiateArrayBuffer(receiveInstantiationResult);
                    });
                });
            }
            else {
                return instantiateArrayBuffer(receiveInstantiationResult);
            }
        }
        if (Module['instantiateWasm']) {
            try {
                var exports = Module['instantiateWasm'](info, receiveInstance);
                return exports;
            }
            catch (e) {
                err('Module.instantiateWasm callback failed with error: ' + e);
                return false;
            }
        }
        instantiateAsync();
        return {};
    }
    function callRuntimeCallbacks(callbacks) {
        while (callbacks.length > 0) {
            var callback = callbacks.shift();
            if (typeof callback == 'function') {
                callback(Module);
                continue;
            }
            var func = callback.func;
            if (typeof func === 'number') {
                if (callback.arg === undefined) {
                    wasmTable.get(func)();
                }
                else {
                    wasmTable.get(func)(callback.arg);
                }
            }
            else {
                func(callback.arg === undefined ? null : callback.arg);
            }
        }
    }
    function emscripten_realloc_buffer(size) {
        try {
            wasmMemory.grow((size - buffer.byteLength + 65535) >>> 16);
            updateGlobalBufferAndViews(wasmMemory.buffer);
            return 1;
        }
        catch (e) { }
    }
    function _emscripten_resize_heap(requestedSize) {
        var oldSize = HEAPU8.length;
        requestedSize = requestedSize >>> 0;
        var maxHeapSize = 2147483648;
        if (requestedSize > maxHeapSize) {
            return false;
        }
        for (var cutDown = 1; cutDown <= 4; cutDown *= 2) {
            var overGrownHeapSize = oldSize * (1 + 0.2 / cutDown);
            overGrownHeapSize = Math.min(overGrownHeapSize, requestedSize + 100663296);
            var newSize = Math.min(maxHeapSize, alignUp(Math.max(requestedSize, overGrownHeapSize), 65536));
            var replacement = emscripten_realloc_buffer(newSize);
            if (replacement) {
                return true;
            }
        }
        return false;
    }
    function _setTempRet0(val) {
    }
    var asmLibraryArg = { a: _emscripten_resize_heap, b: _setTempRet0 };
    Module['___wasm_call_ctors'] = function () {
        return (Module['___wasm_call_ctors'] = Module['asm']['d']).apply(null, arguments);
    };
    Module['_ZSTD_isError'] = function () {
        return (Module['_ZSTD_isError'] = Module['asm']['e']).apply(null, arguments);
    };
    Module['_ZSTD_compressBound'] = function () {
        return (Module['_ZSTD_compressBound'] = Module['asm']['f']).apply(null, arguments);
    };
    Module['_ZSTD_createCCtx'] = function () {
        return (Module['_ZSTD_createCCtx'] = Module['asm']['g']).apply(null, arguments);
    };
    Module['_ZSTD_freeCCtx'] = function () {
        return (Module['_ZSTD_freeCCtx'] = Module['asm']['h']).apply(null, arguments);
    };
    Module['_ZSTD_compress_usingDict'] = function () {
        return (Module['_ZSTD_compress_usingDict'] = Module['asm']['i']).apply(null, arguments);
    };
    Module['_ZSTD_compress'] = function () {
        return (Module['_ZSTD_compress'] = Module['asm']['j']).apply(null, arguments);
    };
    Module['_ZSTD_createDCtx'] = function () {
        return (Module['_ZSTD_createDCtx'] = Module['asm']['k']).apply(null, arguments);
    };
    Module['_ZSTD_freeDCtx'] = function () {
        return (Module['_ZSTD_freeDCtx'] = Module['asm']['l']).apply(null, arguments);
    };
    Module['_ZSTD_getFrameContentSize'] = function () {
        return (Module['_ZSTD_getFrameContentSize'] = Module['asm']['m']).apply(null, arguments);
    };
    Module['_ZSTD_decompress_usingDict'] = function () {
        return (Module['_ZSTD_decompress_usingDict'] = Module['asm']['n']).apply(null, arguments);
    };
    Module['_ZSTD_decompress'] = function () {
        return (Module['_ZSTD_decompress'] = Module['asm']['o']).apply(null, arguments);
    };
    Module['_malloc'] = function () {
        return (Module['_malloc'] = Module['asm']['p']).apply(null, arguments);
    };
    Module['_free'] = function () {
        return (Module['_free'] = Module['asm']['q']).apply(null, arguments);
    };
    var calledRun;
    dependenciesFulfilled = function runCaller() {
        if (!calledRun)
            run();
        if (!calledRun)
            dependenciesFulfilled = runCaller;
    };
    function run(args) {
        if (runDependencies > 0) {
            return;
        }
        preRun();
        if (runDependencies > 0) {
            return;
        }
        function doRun() {
            if (calledRun)
                return;
            calledRun = true;
            Module['calledRun'] = true;
            if (ABORT)
                return;
            initRuntime();
            if (Module['onRuntimeInitialized'])
                Module['onRuntimeInitialized']();
            postRun();
        }
        if (Module['setStatus']) {
            Module['setStatus']('Running...');
            setTimeout(function () {
                setTimeout(function () {
                    Module['setStatus']('');
                }, 1);
                doRun();
            }, 1);
        }
        else {
            doRun();
        }
    }
    Module['run'] = run;
    if (Module['preInit']) {
        if (typeof Module['preInit'] == 'function')
            Module['preInit'] = [Module['preInit']];
        while (Module['preInit'].length > 0) {
            Module['preInit'].pop()();
        }
    }
    Module['init'] = init$1;

    var __awaiter$1 = (undefined && undefined.__awaiter) || function (thisArg, _arguments, P, generator) {
        function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
        return new (P || (P = Promise))(function (resolve, reject) {
            function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
            function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
            function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
            step((generator = generator.apply(thisArg, _arguments || [])).next());
        });
    };
    const initialized = (() => new Promise((resolve) => {
        Module.onRuntimeInitialized = resolve;
    }))();
    const waitInitialized = () => __awaiter$1(void 0, void 0, void 0, function* () {
        yield initialized;
    });

    const isError = (code) => {
        const _isError = Module['_ZSTD_isError'];
        return _isError(code);
    };
    // @See https://github.com/facebook/zstd/blob/12c045f74d922dc934c168f6e1581d72df983388/lib/common/error_private.c#L24-L53
    // export const getErrorName = (code: number): string => {
    //   const _getErrorName = Module.cwrap('ZSTD_getErrorName', 'string', ['number']);
    //   return _getErrorName(code);
    // };

    const getFrameContentSize$1 = (src, size) => {
        const getSize = Module['_ZSTD_getFrameContentSize'];
        return getSize(src, size);
    };
    const decompress = (buf, opts = { defaultHeapSize: 1024 * 1024 }) => {
        const malloc = Module['_malloc'];
        const src = malloc(buf.byteLength);
        Module.HEAP8.set(buf, src);
        const contentSize = getFrameContentSize$1(src, buf.byteLength);
        const size = contentSize === -1 ? opts.defaultHeapSize : contentSize;
        const free = Module['_free'];
        const heap = malloc(size);
        try {
            /*
              @See https://zstd.docsforge.com/dev/api/ZSTD_decompress/
              compressedSize : must be the exact size of some number of compressed and/or skippable frames.
              dstCapacity is an upper bound of originalSize to regenerate.
              If user cannot imply a maximum upper bound, it's better to use streaming mode to decompress data.
              @return: the number of bytes decompressed into dst (<= dstCapacity), or an errorCode if it fails (which can be tested using ZSTD_isError()).
            */
            const _decompress = Module['_ZSTD_decompress'];
            const sizeOrError = _decompress(heap, size, src, buf.byteLength);
            if (isError(sizeOrError)) {
                throw new Error(`Failed to compress with code ${sizeOrError}`);
            }
            // Copy buffer
            // Uint8Array.prototype.slice() return copied buffer.
            const data = new Uint8Array(Module.HEAPU8.buffer, heap, sizeOrError).slice();
            free(heap, size);
            free(src, buf.byteLength);
            return data;
        }
        catch (e) {
            free(heap, size);
            free(src, buf.byteLength);
            throw e;
        }
    };

    const compressBound$1 = (size) => {
        const bound = Module['_ZSTD_compressBound'];
        return bound(size);
    };
    const compress = (buf, level) => {
        const bound = compressBound$1(buf.byteLength);
        const malloc = Module['_malloc'];
        const compressed = malloc(bound);
        const src = malloc(buf.byteLength);
        Module.HEAP8.set(buf, src);
        const free = Module['_free'];
        try {
            /*
              @See https://zstd.docsforge.com/dev/api/ZSTD_compress/
              size_t ZSTD_compress( void* dst, size_t dstCapacity, const void* src, size_t srcSize, int compressionLevel);
              Compresses `src` content as a single zstd compressed frame into already allocated `dst`.
              Hint : compression runs faster if `dstCapacity` >=  `ZSTD_compressBound(srcSize)`.
              @return : compressed size written into `dst` (<= `dstCapacity),
                        or an error code if it fails (which can be tested using ZSTD_isError()).
            */
            const _compress = Module['_ZSTD_compress'];
            const sizeOrError = _compress(compressed, bound, src, buf.byteLength, level !== null && level !== void 0 ? level : 3);
            if (isError(sizeOrError)) {
                throw new Error(`Failed to compress with code ${sizeOrError}`);
            }
            // // Copy buffer
            // // Uint8Array.prototype.slice() return copied buffer.
            const data = new Uint8Array(Module.HEAPU8.buffer, compressed, sizeOrError).slice();
            free(compressed, bound);
            free(src, buf.byteLength);
            return data;
        }
        catch (e) {
            free(compressed, bound);
            free(src, buf.byteLength);
            throw e;
        }
    };

    const getFrameContentSize = (src, size) => {
        const getSize = Module['_ZSTD_getFrameContentSize'];
        return getSize(src, size);
    };
    const createDCtx = () => {
        return Module['_ZSTD_createDCtx']();
    };
    const freeDCtx = (dctx) => {
        return Module['_ZSTD_freeDCtx'](dctx);
    };
    const decompressUsingDict = (dctx, buf, dict, opts = { defaultHeapSize: 1024 * 1024 }) => {
        const malloc = Module['_malloc'];
        const src = malloc(buf.byteLength);
        Module.HEAP8.set(buf, src);
        const pdict = malloc(dict.byteLength);
        Module.HEAP8.set(dict, pdict);
        const contentSize = getFrameContentSize(src, buf.byteLength);
        const size = contentSize === -1 ? opts.defaultHeapSize : contentSize;
        const free = Module['_free'];
        const heap = malloc(size);
        try {
            const _decompress = Module['_ZSTD_decompress_usingDict'];
            const sizeOrError = _decompress(dctx, heap, size, src, buf.byteLength, pdict, dict.byteLength);
            if (isError(sizeOrError)) {
                throw new Error(`Failed to compress with code ${sizeOrError}`);
            }
            // Copy buffer
            // Uint8Array.prototype.slice() return copied buffer.
            const data = new Uint8Array(Module.HEAPU8.buffer, heap, sizeOrError).slice();
            free(heap, size);
            free(src, buf.byteLength);
            free(pdict, dict.byteLength);
            return data;
        }
        catch (e) {
            free(heap, size);
            free(src, buf.byteLength);
            free(pdict, dict.byteLength);
            throw e;
        }
    };

    const compressBound = (size) => {
        const bound = Module['_ZSTD_compressBound'];
        return bound(size);
    };
    const createCCtx = () => {
        return Module['_ZSTD_createCCtx']();
    };
    const freeCCtx = (cctx) => {
        return Module['_ZSTD_freeCCtx'](cctx);
    };
    const compressUsingDict = (cctx, buf, dict, level) => {
        const bound = compressBound(buf.byteLength);
        const malloc = Module['_malloc'];
        const compressed = malloc(bound);
        const src = malloc(buf.byteLength);
        Module.HEAP8.set(buf, src);
        // Setup dict
        const pdict = malloc(dict.byteLength);
        Module.HEAP8.set(dict, pdict);
        const free = Module['_free'];
        try {
            /*
              @See https://zstd.docsforge.com/dev/api/ZSTD_compress_usingDict/
              size_t ZSTD_compress_usingDict(ZSTD_CCtx* cctx,
                                 void* dst, size_t dstCapacity,
                                 const void* src, size_t srcSize,
                                 const void* dict, size_t dictSize,
                                 int compressionLevel)
            */
            const _compress = Module['_ZSTD_compress_usingDict'];
            const sizeOrError = _compress(cctx, compressed, bound, src, buf.byteLength, pdict, dict.byteLength, level !== null && level !== void 0 ? level : 3);
            if (isError(sizeOrError)) {
                throw new Error(`Failed to compress with code ${sizeOrError}`);
            }
            // // Copy buffer
            // // Uint8Array.prototype.slice() return copied buffer.
            const data = new Uint8Array(Module.HEAPU8.buffer, compressed, sizeOrError).slice();
            free(compressed, bound);
            free(src, buf.byteLength);
            free(pdict, dict.byteLength);
            return data;
        }
        catch (e) {
            free(compressed, bound);
            free(src, buf.byteLength);
            free(pdict, dict.byteLength);
            throw e;
        }
    };

    var __awaiter = (undefined && undefined.__awaiter) || function (thisArg, _arguments, P, generator) {
        function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
        return new (P || (P = Promise))(function (resolve, reject) {
            function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
            function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
            function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
            step((generator = generator.apply(thisArg, _arguments || [])).next());
        });
    };
    const init = (path) => __awaiter(void 0, void 0, void 0, function* () {
        // @ts-ignore
        const url = new URL(`./zstd.wasm`, (typeof document === 'undefined' && typeof location === 'undefined' ? require('u' + 'rl').pathToFileURL(__filename).href : typeof document === 'undefined' ? location.href : (_documentCurrentScript && _documentCurrentScript.tagName.toUpperCase() === 'SCRIPT' && _documentCurrentScript.src || new URL('zstd-bundle.js', document.baseURI).href))).href;
        Module['init'](path !== null && path !== void 0 ? path : url);
        yield waitInitialized();
    });

    exports.compress = compress;
    exports.compressUsingDict = compressUsingDict;
    exports.createCCtx = createCCtx;
    exports.createDCtx = createDCtx;
    exports.decompress = decompress;
    exports.decompressUsingDict = decompressUsingDict;
    exports.freeCCtx = freeCCtx;
    exports.freeDCtx = freeDCtx;
    exports.init = init;

}));
