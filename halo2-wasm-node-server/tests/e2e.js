const assert = require('assert');
const fs = require('fs');
const server = require('../src/server');
const client = require('./client');

const addr = '0.0.0.0:14002'
describe('VmRuntime Service', function() {
  before(function() {
    server.startGrpcServer(addr);
  });

  after(function() {
    server.stopGrpcServer();
  });

  it('should create successfully', function(done) {
    let code;
    try {
        const data = fs.readFileSync('./tests/10001.json', 'utf8');
        const jsonData = JSON.parse(data);
        code = jsonData.code; 
        console.log(data);
    } catch (err) {
        assert.fail('Error occurred: ' + err.message);
    }
    client.create(addr, {projectID: 10001, content: code}, function(err, response) {
        assert.strictEqual(err, null);
        done();
    });
  });

  it('should execute operator successfully', function(done) {
    client.executeOperator(addr, {projectID: 10001, taskID: 0, clientID: "clientID", sequencerSignature: "sequencerSignature", datas: ['{\"private_a\": 3, \"private_b\": 5}']}, function(err, response) {
        assert.strictEqual(err, null);
        assert.notStrictEqual(response.result.toString(), null);
        done();
    });
  });

  it('project not found', function(done) {
    client.executeOperator(addr, {projectID: 99999,taskID: 0, clientID: "clientID", sequencerSignature: "sequencerSignature", datas: ['{\"private_a\": 3, \"private_b\": 5}']}, function(err, response) {
        assert.ok(err.message.includes("projectID '99999' does not exist in the halo2 vm."));
        done();
    });
  });
});