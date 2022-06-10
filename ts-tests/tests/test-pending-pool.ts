import { expect } from "chai";

import { createAndFinalizeBlock, customRequest, describeWithFrontier } from "./util";

describeWithFrontier("Frontier RPC (Pending Pool)", (context) => {
	const GENESIS_ACCOUNT = "0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac";
	const GENESIS_ACCOUNT_PRIVATE_KEY = "0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133";

	// Solidity: contract test { function multiply(uint a) public pure returns(uint d) {return a * 7;}}
	const TEST_CONTRACT_BYTECODE =
		"0x6080604052348015600f57600080fd5b5060ae8061001e6000396000f3fe6080604052348015600f57600080fd5b506004361060285760003560e01c8063c6888fa114602d575b600080fd5b605660048036036020811015604157600080fd5b8101908080359060200190929190505050606c565b6040518082815260200191505060405180910390f35b600060078202905091905056fea265627a7a72315820f06085b229f27f9ad48b2ff3dd9714350c1698a37853a30136fa6c5a7762af7364736f6c63430005110032";
	const FIRST_CONTRACT_ADDRESS = "0xc2bf5f29a4384b1ab0c063e1c666f02121b6084a";

	it("should return a pending transaction", async function () {
		this.timeout(15000);
		const tx = await context.web3.eth.accounts.signTransaction(
			{
				from: GENESIS_ACCOUNT,
				data: TEST_CONTRACT_BYTECODE,
				value: "0x00",
				gasPrice: "0x3B9ACA00",
				gas: "0x100000",
			},
			GENESIS_ACCOUNT_PRIVATE_KEY
		);

		const tx_hash = (await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction])).result;

		const pending_transaction = (await customRequest(context.web3, "eth_getTransactionByHash", [tx_hash])).result;
		// pending transactions do not know yet to which block they belong to
		expect(pending_transaction).to.include({
			blockNumber: null,
			hash: tx_hash,
			publicKey: "0x509540919faacf9ab52146c9aa40db68172d83777250b28e4679176e49ccdd9fa213197dc0666e85529d6c9dda579c1295d61c417f01505765481e89a4016f02",
			r: "0x533e0f65a9cf3d0c8e5dd3f51744f66c7df3171608d104bcceaceab922def57d",
			s: "0x184d6b08b2f4f1e9b86a75a83a8301bb6d00597e1d38eeece0c9594bf14c0bcf",
			v: "0x163",
		});

		await createAndFinalizeBlock(context.web3);

		const processed_transaction = (await customRequest(context.web3, "eth_getTransactionByHash", [tx_hash])).result;
		expect(processed_transaction).to.include({
			hash: tx_hash,
			publicKey: "0x509540919faacf9ab52146c9aa40db68172d83777250b28e4679176e49ccdd9fa213197dc0666e85529d6c9dda579c1295d61c417f01505765481e89a4016f02",
			r: "0x533e0f65a9cf3d0c8e5dd3f51744f66c7df3171608d104bcceaceab922def57d",
			s: "0x184d6b08b2f4f1e9b86a75a83a8301bb6d00597e1d38eeece0c9594bf14c0bcf",
			v: "0x163",
		});
	});
});

describeWithFrontier("Frontier RPC (Pending Transaction Count)", (context) => {
	const GENESIS_ACCOUNT = "0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac";
	const GENESIS_ACCOUNT_PRIVATE_KEY = "0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133";
	const TEST_ACCOUNT = "0x1111111111111111111111111111111111111111";

	it("should return pending transaction count", async function () {
		this.timeout(15000);

		// nonce should be 0
		expect(await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT, 'latest')).to.eq(0);

		var nonce = 0;
		let sendTransaction = async () => {
			const tx = await context.web3.eth.accounts.signTransaction(
				{
					from: GENESIS_ACCOUNT,
					to: TEST_ACCOUNT,
					value: "0x200", // Must be higher than ExistentialDeposit (500)
					gasPrice: "0x3B9ACA00",
					gas: "0x100000",
					nonce: nonce,
				},
				GENESIS_ACCOUNT_PRIVATE_KEY
			);
			nonce = nonce + 1;
			return (await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction])).result
		};

		{
			const pending_transaction_count = (await customRequest(context.web3, "eth_getBlockTransactionCountByNumber", ['pending'])).result;
			expect(pending_transaction_count).to.eq('0x0');
		}

		// block 1 should have 1 transaction
		await sendTransaction();
		{
			const pending_transaction_count = (await customRequest(context.web3, "eth_getBlockTransactionCountByNumber", ['pending'])).result;
			expect(pending_transaction_count).to.eq('0x1');
		}

		await createAndFinalizeBlock(context.web3);

		{
			const pending_transaction_count = (await customRequest(context.web3, "eth_getBlockTransactionCountByNumber", ['pending'])).result;
			expect(pending_transaction_count).to.eq('0x0');
			const processed_transaction_count = (await customRequest(context.web3, "eth_getBlockTransactionCountByNumber", [1])).result;
			expect(processed_transaction_count).to.eq('0x1');
		}

		// block 2 should have 5 transactions
		for (var _ of Array(5)) {
			await sendTransaction();
		}

		{
			const pending_transaction_count = (await customRequest(context.web3, "eth_getBlockTransactionCountByNumber", ['pending'])).result;
			expect(pending_transaction_count).to.eq('0x5');
		}

		await createAndFinalizeBlock(context.web3);

		{
			const pending_transaction_count = (await customRequest(context.web3, "eth_getBlockTransactionCountByNumber", ['pending'])).result;
			expect(pending_transaction_count).to.eq('0x0');
			const processed_transaction_count = (await customRequest(context.web3, "eth_getBlockTransactionCountByNumber", [2])).result;
			expect(processed_transaction_count).to.eq('0x5');
		}
	});
});
