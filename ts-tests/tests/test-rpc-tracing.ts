import { expect } from "chai";
import { step } from "mocha-steps";
import { AbiItem } from "web3-utils";
import { createAndFinalizeBlock, describeWithFrontier, customRequest } from "./util";

import TestERC20 from "../build/contracts/TestERC20.json";

describeWithFrontier("ETH RPC(Tracing For ERC20)", (context) => {
	const GENESIS_ACCOUNT = "0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac";
	const GENESIS_ACCOUNT_BALANCE = "1152921504606846976";
	const GENESIS_ACCOUNT_PRIVATE_KEY = "0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133";
	const TEST_ACCOUNT = "0x1111111111111111111111111111111111111111";
	const TEST_CONTRACT_BYTECODE = TestERC20.bytecode;
	const TEST_CONTRACT_ABI = TestERC20.abi as AbiItem[];
	let CONTRACT_ADDRESS = "0x00"; // Those test are ordered. In general this should be avoided, but due to the time it takes	// to spin up a frontier node, it saves a lot of time.

	before("create the contract", async function () {
		this.timeout(15000);
		const tx = await context.web3.eth.accounts.signTransaction(
			{
				from: GENESIS_ACCOUNT,
				data: TEST_CONTRACT_BYTECODE,
				value: "0x00",
				gasPrice: "0x3B9ACA00",
				gas: "0x1000000",
			},
			GENESIS_ACCOUNT_PRIVATE_KEY
		);
		await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
		await createAndFinalizeBlock(context.web3);

		// set the contract address
		let receipt0 = await context.web3.eth.getTransactionReceipt(tx.transactionHash);

		CONTRACT_ADDRESS = receipt0.contractAddress;
	});

	step("Get tracing information for ERC20 transfer", async function () {
		this.timeout(15000);

		const contract = new context.web3.eth.Contract(TEST_CONTRACT_ABI, CONTRACT_ADDRESS, {
			from: GENESIS_ACCOUNT,
			gasPrice: "0x3B9ACA00",
			gas: 100000,
		});

		const tx = await context.web3.eth.accounts.signTransaction(
			{
				from: GENESIS_ACCOUNT,
				to: CONTRACT_ADDRESS,
				data: contract.methods.transfer(TEST_ACCOUNT, 1000).encodeABI(),
				value: "0x00",
				gasPrice: "0x3B9ACA00",
				gas: "0x500000",
			},
			GENESIS_ACCOUNT_PRIVATE_KEY
		);
		await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
		await createAndFinalizeBlock(context.web3);

		expect(await contract.methods.balanceOf(TEST_ACCOUNT).call()).to.be.eq("1000");

		let res = await context.web3["trace"].transaction(tx.transactionHash);

		// console.log(JSON.stringify(res, null, 2));

		expect(res.length).to.eq(1);

		let tracing_res = res[0];

		expect(tracing_res["type"]).to.eq("call");
		expect(tracing_res["action"]["from"]).to.eq("0xf24ff3a9cf04c71dbc94d0b566f7a27b94566cac");
		expect(tracing_res["result"]["gasUsed"]).to.eq("0xc189");

		let block_res = await context.web3["trace"].block("latest");

		expect(block_res.length).to.eq(1);

		let block_tracing_res = block_res[0];

		expect(block_tracing_res["type"]).to.eq("call");
		expect(block_tracing_res["action"]["from"]).to.eq("0xf24ff3a9cf04c71dbc94d0b566f7a27b94566cac");
		expect(block_tracing_res["result"]["gasUsed"]).to.eq("0xc189");
	});
});
