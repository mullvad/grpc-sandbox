const protoLoader = require('@grpc/proto-loader');
const grpc = require('@grpc/grpc-js');

const path = process.platform === 'win32' ? '\\\\.\\pipe\\helloworld' : 'unix:///tmp/helloworld';

protoLoader
  .load('../proto/echo.proto')
  .then((packageDefinition) => {
    const { Echo } = grpc.loadPackageDefinition(packageDefinition).grpc.examples.echo;
    // const { Echo, EchoRequest, EchoResponse } = ...

    const credentials = grpc.credentials.createInsecure();
    const client = new Echo(path, credentials);

    client.unaryEcho({ message: 'hello' }, function(_, response) {
      console.log(response);

      const call = client.serverStreamingEcho({ message: 'HELLO2' });
      call.on('data', console.log);
    });
  })
  .catch(console.error);
