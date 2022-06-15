import { expect } from "chai";

import { createAndFinalizeBlock, customRequest, describeWithFrontier } from "./util";

describeWithFrontier("Frontier RPC (Constructor Revert)", (context) => {
	const GENESIS_ACCOUNT = "0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac";
	const GENESIS_ACCOUNT_PRIVATE_KEY =
		"0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133";

	// ```
	// pragma solidity >=0.4.22 <0.7.0;
	//
	// contract WillFail {
	//		 constructor() public {
	//				 require(false);
	//		 }
	// }
	// ```
	const FAIL_BYTECODE = '6080604052348015600f57600080fd5b506000601a57600080fd5b603f8060276000396000f3fe6080604052600080fdfea26469706673582212209f2bb2a4cf155a0e7b26bd34bb01e9b645a92c82e55c5dbdb4b37f8c326edbee64736f6c63430006060033';
	const GOOD_BYTECODE = '6080604052348015600f57600080fd5b506001601a57600080fd5b603f8060276000396000f3fe6080604052600080fdfea2646970667358221220c70bc8b03cdfdf57b5f6c4131b836f9c2c4df01b8202f530555333f2a00e4b8364736f6c63430006060033';

	it("should provide a tx receipt after successful deployment", async function () {
		this.timeout(15000);
		const GOOD_TX_HASH = '0x8ff277d5071414d7952ac85fc8455aaeacc2af6b5c56ee53df71d6fdece42e5a';

		const tx = await context.web3.eth.accounts.signTransaction(
			{
				from: GENESIS_ACCOUNT,
				data: GOOD_BYTECODE,
				value: "0x00",
				gasPrice: "0x3B9ACA00",
				gas: "0x100000",
			},
			GENESIS_ACCOUNT_PRIVATE_KEY
		);

		expect(
			await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction])
		).to.deep.equal({
			id: 1,
			jsonrpc: "2.0",
			result: GOOD_TX_HASH,
		});

		// Verify the receipt exists after the block is created
		await createAndFinalizeBlock(context.web3);
		const receipt = await context.web3.eth.getTransactionReceipt(GOOD_TX_HASH);
		expect(receipt).to.include({
			contractAddress: '0xc01Ee7f10EA4aF4673cFff62710E1D7792aBa8f3',
			cumulativeGasUsed: 67231,
			from: '0xf24ff3a9cf04c71dbc94d0b566f7a27b94566cac',
			gasUsed: 67231,
			to: null,
			transactionHash: GOOD_TX_HASH,
			transactionIndex: 0,
			status: true
		});
	});

	it("should provide a tx receipt after failed deployment", async function () {
		this.timeout(15000);
		// Transaction hash depends on which nonce we're using
		//const FAIL_TX_HASH = '0x89a956c4631822f407b3af11f9251796c276655860c892919f848699ed570a8d'; //nonce 1
		const FAIL_TX_HASH = '0x9b175cc21fb8b9c2fdbab97d5d6c1c9b2d54dcd4c299b33ac559b8c90a809236'; //nonce 2

		const tx = await context.web3.eth.accounts.signTransaction(
			{
				from: GENESIS_ACCOUNT,
				data: FAIL_BYTECODE,
				value: "0x00",
				gasPrice: "0x3B9ACA00",
				gas: "0x100000",
			},
			GENESIS_ACCOUNT_PRIVATE_KEY
		);

		expect(
			await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction])
		).to.deep.equal({
			id: 1,
			jsonrpc: "2.0",
			result: FAIL_TX_HASH,
		});

		await createAndFinalizeBlock(context.web3);
		const receipt = await context.web3.eth.getTransactionReceipt(FAIL_TX_HASH);
		expect(receipt).to.include({
			contractAddress: '0x970951a12F975E6762482ACA81E57D5A2A4e73F4',
			cumulativeGasUsed: 54600,
			from: '0xf24ff3a9cf04c71dbc94d0b566f7a27b94566cac',
			gasUsed: 54600,
			to: null,
			transactionHash: FAIL_TX_HASH,
			transactionIndex: 0,
			status: false
		});
	});
});
