FROM node:14

WORKDIR /halo2-wasm-server

COPY ./ .

RUN npm install

CMD [ "node", "src/server.js" ]