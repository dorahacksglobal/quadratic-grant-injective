import {
  MsgSend,
  ChainRestAuthApi,
  ChainRestTendermintApi,
  BaseAccount,
  ChainGrpcBankApi,
  ChainGrpcWasmApi,
  MsgBroadcasterWithPk,
  PrivateKey,
  MsgExecuteContract,
  TxResponse,
  IndexerRestExplorerApi,
} from "@injectivelabs/sdk-ts";

import {
  DEFAULT_BLOCK_TIMEOUT_HEIGHT,
  BigNumberInBase,
} from "@injectivelabs/utils";

import { Network, getNetworkEndpoints } from "@injectivelabs/networks";

const privateKeyHash =
  "6a9d3ca63d6832cfd283d17837371b0328a46bf2566752e188c407d6ddb5007c";
const privateKey = PrivateKey.fromHex(privateKeyHash);
const injectiveAddress = privateKey.toBech32();
const publicKey = privateKey.toPublicKey().toBase64();
const contractAddress = "inj1m573ael4haga2w0vgawrc7mdt2g994tml07wgp";

const network = Network.TestnetK8s;
const endpoints = getNetworkEndpoints(network);
console.log(endpoints);

const chainGrpcBankApi = new ChainGrpcBankApi(endpoints.grpc);
const chainRestAuthApi = new ChainRestAuthApi(endpoints.rest);
const chainGrpcWasmApi = new ChainGrpcWasmApi(endpoints.grpc);
const chainRestTendermintApi = new ChainRestTendermintApi(endpoints.rest);
const indexerGrpcExplorerApi = new IndexerRestExplorerApi(endpoints.indexer);

async function main() {
  // await account(chainGrpcBankApi, chainRestAuthApi);
  // await block(chainRestTendermintApi);
  // await sendCoin();

  // const contractInfo = await chainGrpcWasmApi.fetchContractInfo(
  //   contractAddress
  // );
  // console.log({ contractInfo });
  const contractTx = await indexerGrpcExplorerApi.fetchContractTransactions({
    contractAddress,
    params: {
      limit: 10,
    },
  });
  console.log(contractTx);

  let txExecHash: TxResponse;

  // // Start Round
  // txExecHash = await execute({
  //   start_round: {
  //     tax_adjustment_multiplier: 10,
  //     donation_denom: "inj",
  //     voting_unit: "1000000000000000000",
  //     fund: "4000",
  //   },
  // });
  // console.log("start_round: ", txExecHash.txHash);

  // Batch Upload Projects
  // txExecHash = await execute({
  //   batch_upload_project: { round_id: 1, owner_addresses: ["3", "4", "5", "6", "7", "8", "9", "10"] },
  // });
  // console.log("batch_upload_project: ", txExecHash.txHash);

  // Weighted Batch Vote 1
  // txExecHash = await execute(
  //   {
  //     weighted_batch_vote: {
  //       round_id: 1,
  //       project_ids: [1],
  //       amounts: ["160000"],
  //       vcdora: 0,
  //       timestamp: 0,
  //       recid: 0,
  //       sig: [],
  //     },
  //   },
  //   { denom: "inj", amount: "160000" }
  // );
  // console.log("weighted_batch_vote1: ", txExecHash.txHash);

  // // Weighted Batch Vote 2
  // txExecHash = await execute(
  //   {
  //     weighted_batch_vote: {
  //       round_id: 1,
  //       project_ids: [1],
  //       amounts: ["90000"],
  //       vcdora: 0,
  //       timestamp: 0,
  //       recid: 0,
  //       sig: [],
  //     },
  //   },
  //   { denom: "inj", amount: "90000" }
  // );
  // console.log("weighted_batch_vote2: ", txExecHash.txHash);

  // // Weighted Batch Vote 3
  // txExecHash = await execute({
  //   weighted_batch_vote: {
  //     round_id: 1,
  //     project_votes: [[2, "160000"]],
  //     vcdora: 123143400,
  //   },
  // });
  // console.log("weighted_batch_vote3: ", txExecHash.txHash);

  const roundInfo = await query({ round: { round_id: 1 } });
  console.log("roundInfo: ", roundInfo);
}

async function account(
  chainGrpcBankApi: ChainGrpcBankApi,
  chainRestAuthApi: ChainRestAuthApi
) {
  /** Account Details **/
  const balances = await chainGrpcBankApi.fetchBalances(injectiveAddress);
  console.log({ balances: balances.balances });

  const accountDetailsResponse = await chainRestAuthApi.fetchAccount(
    injectiveAddress
  );
  const baseAccount = BaseAccount.fromRestApi(accountDetailsResponse);
  const accountDetails = baseAccount.toAccountDetails();
  console.log({ accountDetails });
}

async function block(chainRestTendermintApi: ChainRestTendermintApi) {
  /** Block Details */
  const latestBlock = await chainRestTendermintApi.fetchLatestBlock();
  const latestHeight = latestBlock.header.height;
  const timeoutHeight = new BigNumberInBase(latestHeight).plus(
    DEFAULT_BLOCK_TIMEOUT_HEIGHT
  );
  console.log({ latestBlock, latestHeight, timeoutHeight });
}

async function sendCoin() {
  /** Send Coin */
  const amount = {
    amount: new BigNumberInBase(0.01).toWei().toFixed(),
    denom: "inj",
  };
  const msg = MsgSend.fromJSON({
    amount,
    srcInjectiveAddress: injectiveAddress,
    dstInjectiveAddress: injectiveAddress,
  });

  const txHash = await new MsgBroadcasterWithPk({
    privateKey: privateKeyHash,
    network,
    endpoints,
  }).broadcast({
    msgs: msg,
    injectiveAddress,
  });
  console.log({ txHash });
}

async function query(msg: object) {
  // `injectived query wasm contract-state smart`
  const queryData = Buffer.from(JSON.stringify(msg)).toString("base64");
  const wasmQuery = await chainGrpcWasmApi.fetchSmartContractState(
    contractAddress,
    queryData
  );
  const resp = Buffer.from(wasmQuery.data as String, "base64").toString();
  return JSON.parse(resp);
}

async function execute(
  msg: object,
  funds?: {
    denom: string;
    amount: string;
  }
) {
  const msgExec = MsgExecuteContract.fromJSON({
    sender: injectiveAddress,
    contractAddress,
    funds,
    msg,
  });
  const txExecHash = await new MsgBroadcasterWithPk({
    privateKey: privateKeyHash,
    network,
    endpoints,
  }).broadcast({
    msgs: msgExec,
    injectiveAddress,
  });
  return txExecHash;
}

main();
