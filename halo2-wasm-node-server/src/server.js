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
    const taskID = call.request.taskID;
    const clientID = call.request.clientID;
    const sequencerSign = call.request.sequencerSign;
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

    const result = wasm.prove(projectID, taskID, clientID, sequencerSign, JSON.stringify(datas));
    // convert result to bytes
    let resultBytes = new Uint8Array(result.length);
    for (var i = 0; i < result.length; i++) {
        resultBytes[i] = result.charCodeAt(i);
    }

    callback(null, { result: resultBytes });
}

let server;
function startGrpcServer(addr) {
    server = new grpc.Server();
    server.addService(vmRuntime.VmRuntime.service, {
        create: create,
        executeOperator: executeOperator,
    });
    server.bindAsync(addr, grpc.ServerCredentials.createInsecure(), () => {
        server.start();
        console.log('Server running at http://' + addr);
    });
}

function stopGrpcServer() {
    server.forceShutdown();
}

module.exports = {
    startGrpcServer,
    stopGrpcServer
};
  
if (require.main === module) {
    startGrpcServer('0.0.0.0:4001');
}