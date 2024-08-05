const grpc = require('@grpc/grpc-js');
const protoLoader = require('@grpc/proto-loader');
const zlib = require('zlib');

const wasm = require("./wasm_instance")

const PROTO_PATH = '../proto/vm_runtime.proto'

const heapdump = require('heapdump');
const v8 = require('v8');
const heapLimit1GB = 1 * 1024 * 1024 * 1024;
const heapLimit2GB = 2 * 1024 * 1024 * 1024;
let snapshot1GBTaken = false;
let snapshot2GBTaken = false;

function checkMemoryUsage() {
    const usedHeap = v8.getHeapStatistics().used_heap_size;

    if (usedHeap > heapLimit1GB && !snapshot1GBTaken) {
        const filename = `./heapdump-1GB-${Date.now()}.heapsnapshot`;
        heapdump.writeSnapshot(filename, (err) => {
            if (err) {
                console.error('Heapdump for 1GB failed:', err);
            } else {
                console.log('Heapdump for 1GB written to', filename);
                snapshot1GBTaken = true;
            }
        });
    }

    if (usedHeap > heapLimit2GB && !snapshot2GBTaken) {
        const filename = `./heapdump-2GB-${Date.now()}.heapsnapshot`;
        heapdump.writeSnapshot(filename, (err) => {
            if (err) {
                console.error('Heapdump for 2GB failed:', err);
            } else {
                console.log('Heapdump for 2GB written to', filename);
                snapshot2GBTaken = true;
            }
        });
    }
}


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

function execute(call, callback) {
    const projectID = call.request.projectID;
    const taskID = call.request.taskID;
    const clientID = call.request.clientID;
    const sequencerSignature = call.request.sequencerSignature;
    const datas = call.request.datas;

    console.log('executor vm with projectID %d', projectID)

    if (!contentMap.has(projectID)) {
        console.log("projectID '%d' does not exist in the map.", projectID);
        return callback(new Error(`projectID '${projectID}' does not exist in the halo2 vm.`));
    }

    const content = contentMap.get(projectID);
    contentMap.delete(projectID);

    const buffer = Buffer.from(content, 'hex');
    let bytes = zlib.inflateSync(buffer);

    wasm.setWasmBytes(bytes);
    wasm.initWasmInstance();

    const result = wasm.prove(projectID, taskID, clientID, sequencerSignature, JSON.stringify(datas));

    callback(null, { result: result });
}

let server;
function startGrpcServer(addr) {
    server = new grpc.Server();
    server.addService(vmRuntime.VmRuntime.service, {
        create: create,
        execute: execute,
    });
    server.bindAsync(addr, grpc.ServerCredentials.createInsecure(), () => {
        server.start();
        console.log('Server running at http://' + addr);
    });

    setInterval(checkMemoryUsage, 60000);
}

function stopGrpcServer() {
    server.forceShutdown();
}

module.exports = {
    startGrpcServer,
    stopGrpcServer
};

if (require.main === module) {
    startGrpcServer('0.0.0.0:4002');
}