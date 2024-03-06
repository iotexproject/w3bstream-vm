const grpc = require('@grpc/grpc-js');
const protoLoader = require('@grpc/proto-loader');
const zlib = require('zlib');

const wasm = require("./wasm_instance")

const PROTO_PATH = '../proto/vm_runtime.proto'

const packageDefinition = protoLoader.loadSync(PROTO_PATH, {
    keepCase: true,
    longs: String,
    enums: String,
    defaults: true,
    oneofs: true
});

const vmRuntime = grpc.loadPackageDefinition(packageDefinition).vm_runtime;

const contentMap = new Map();

function create(call, callback) {
    const projectID = call.request.projectID;
    const content = call.request.content;

    console.log('create vm with projectID %d', projectID)

    contentMap.set(projectID, content);

    callback(null, {});
}

function executeOperator(call, callback) {
    const projectID = call.request.projectID;
    const datas = call.request.datas;

    console.log('executor vm with projectID %d', projectID)

    if (!contentMap.has(projectID)) {
        console.log("projectID '%d' does not exist in the map.", projectID);
        return callback(new Error(`projectID '${projectID}' does not exist in the halo2 vm.`));
    }

    const content = contentMap.get(projectID);

    const buffer = Buffer.from(content, 'hex');
    let bytes = zlib.inflateSync(buffer);

    wasm.setWasmBytes(bytes);
    wasm.initWasmInstance();

    let result = wasm.prove(JSON.stringify(datas));
    // check hex string
    if (!isHexadecimal(result)) {
        console.log("convert result to hex string.");
        result = Buffer.from(result, 'utf8').toString('hex');
    }
    // convert result to bytes
    let resultBytes = new Uint8Array(result.length);
    for (var i = 0; i < result.length; i++) {
        resultBytes[i] = result.charCodeAt(i);
    }

    callback(null, { result: resultBytes });
}

function isHexadecimal(str) {
    var regexp = /^[0-9a-fA-F]+$/;
    return regexp.test(str);
}

function startGrpcServer() {
    const server = new grpc.Server();
    server.addService(vmRuntime.VmRuntime.service, {
        create: create,
        executeOperator: executeOperator,
    });
    server.bindAsync('0.0.0.0:4001', grpc.ServerCredentials.createInsecure(), () => {
        server.start();
        console.log('Server running at http://0.0.0.0:4001');
    });
}

startGrpcServer()