const grpc = require('@grpc/grpc-js');
const protoLoader = require('@grpc/proto-loader');
const PROTO_PATH = '../proto/vm_runtime.proto'
const fs = require('fs');

const packageDefinition = protoLoader.loadSync(PROTO_PATH, {
    keepCase: true,
    longs: String,
    enums: String,
    defaults: true,
    oneofs: true
});

const vmRuntime = grpc.loadPackageDefinition(packageDefinition).vm_runtime;

function create(addr, request, callback) {
    const client = new vmRuntime.VmRuntime(addr,
                                           grpc.credentials.createInsecure());
    client.create(request, callback);                                           
  }
  
function execute(addr, request, callback) {
    const client = new vmRuntime.VmRuntime(addr,
                                           grpc.credentials.createInsecure());
    client.execute(request, callback);
}

if (require.main === module) {
    let code;
    try {
        const data = fs.readFileSync('./tests/10001.json', 'utf8');
        const jsonData = JSON.parse(data);
        code = jsonData.code; 
    } catch (err) {
        console.error(err);
    }

    create('0.0.0.0:4001', {projectID: 1, content: code}, function(err, response) {
        if (err) {
            console.error('Error during create:', err);
            return; 
        }

        console.log('create vm instance:', response);

        execute('0.0.0.0:4001', {projectID: 1, datas: ['{\"private_a\": 3, \"private_b\": 5}']}, function(err, response) {
            if (err) {
                console.error('Error during Execute:', err);
                return;
            }
            console.log('Execute Response:', response.result.toString());
        })
    });
}

module.exports = {
    create,
    execute
};