const protoLoader = require('@grpc/proto-loader');
const grpc = require('@grpc/grpc-js');
const net = require('net');

// A path that is parsable by grpc-js' resolvers is needed.
const fakePath = 'unix://fakepath';
const path = process.platform === 'win32' ? '\\\\.\\pipe\\helloworld' : '/tmp/helloworld';

function createConnection() {
  return net.createConnection({ path });
}

protoLoader
  .load('../proto/echo.proto')
  .then((packageDefinition) => {
    const { Echo } = grpc.loadPackageDefinition(packageDefinition).grpc.examples.echo;
    // const { Echo, EchoRequest, EchoResponse } = ...

    const credentials = grpc.credentials.createInsecure();
    credentials._getConnectionOptions = () => ({ createConnection });

    const client = new Echo(fakePath, credentials);

    client.unaryEcho({ message: 'hello' }, function(_, response) {
      console.log(response);

      const call = client.serverStreamingEcho({ message: 'HELLO2' });
      call.on('data', console.log);
    });
  })
  .catch(console.error);
