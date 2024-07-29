FROM node:21

WORKDIR /halo2-wasm-server

COPY proto /halo2-wasm-server/proto

COPY halo2-wasm-node-server server/

WORKDIR /halo2-wasm-server/server

RUN npm install
RUN npm install -g clinic

CMD [ "clinic", "doctor -- node src/server.js" ]