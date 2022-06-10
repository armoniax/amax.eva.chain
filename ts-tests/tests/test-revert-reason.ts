import { expect } from "chai";

import ExplicitRevertReason from "../build/contracts/ExplicitRevertReason.json"
import { createAndFinalizeBlock, customRequest, describeWithFrontier } from "./util";
import { AbiItem } from "web3-utils";

describeWithFrontier("Frontier RPC (Revert Reason)", (context) => {

	let contractAddress;

	const GENESIS_ACCOUNT = "0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac";
	const GENESIS_ACCOUNT_PRIVATE_KEY = "0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133";
	const REVERT_W_MESSAGE_BYTECODE = ExplicitRevertReason.bytecode;

	const TEST_CONTRACT_ABI= ExplicitRevertReason.abi as AbiItem[];

	before("create the contract", async function () {
		this.timeout(15000);
		const tx = await context.web3.eth.accounts.signTransaction(
			{
				from: GENESIS_ACCOUNT,
				data: REVERT_W_MESSAGE_BYTECODE,
				value: "0x00",
				gasPrice: "0x3B9ACA00",
				gas: "0x100000",
			},
			GENESIS_ACCOUNT_PRIVATE_KEY
		);
		const r = await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
		await createAndFinalizeBlock(context.web3);
		const receipt = await context.web3.eth.getTransactionReceipt(r.result);
		contractAddress = receipt.contractAddress;
	});

	it("should fail with revert reason", async function () {
		const contract = new context.web3.eth.Contract(TEST_CONTRACT_ABI, contractAddress, {
			from: GENESIS_ACCOUNT,
			gasPrice: "0x3B9ACA00",
		});
		try {
			await contract.methods.max10(30).call();
		} catch (error) {
			expect(error.message).to.be.eq(
				"Returned error: VM Exception while processing transaction: revert Value must not be greater than 10."
			);
		}
	});
});
